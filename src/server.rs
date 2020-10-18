use crate::internal::*;

pub struct Handle<T: Sized>(std::sync::Arc<std::sync::Mutex<T>>);

impl<T: Sized> Handle<T> {
    pub fn lock(&self) -> std::sync::MutexGuard<T> {
        self.0.lock().unwrap()
    }
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

pub struct Server<T> {
    data: T,
}

pub type Response = Result<Value, Error>;

pub struct Func<T>(pub std::sync::Arc<dyn Fn(&mut T, &mut Client, Command) -> Response>);

pub struct Commands<'a, T>(std::collections::BTreeMap<&'a str, Func<T>>);

#[macro_export]
macro_rules! commands {
    ($($x:ident),*$(,)?) => {
        fn commands(&self) -> &[&str] {
            &[$(
                stringify!($x),
            )*]
        }

        fn command(&self, name: &str) -> Option<Func<Self>> {
            match name {
                $(
                    stringify!($x) => Some($crate::Func(std::sync::Arc::new(Self::$x))),
                )*
                _ => None
            }
        }
    }
}

impl<'a, T> Commands<'a, T> {
    pub fn new() -> Self {
        Commands(Default::default())
    }

    pub fn add<F: 'static + Fn(&mut T, &mut Client, Command) -> Response>(
        mut self,
        key: &'a str,
        f: F,
    ) -> Self {
        self.0.insert(key, Func(std::sync::Arc::new(f)));
        self
    }

    pub fn get(&self, name: &str) -> Option<&Func<T>> {
        self.0.get(name)
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[async_trait::async_trait]
pub trait Handler: Send + Sized {
    fn command(&self, name: &str) -> Option<Func<Self>>;
    fn commands(&self) -> &[&str];

    fn password_required(&self) -> bool;
    fn _check_password(&self, _username: &str, _password: &str) -> bool;

    fn handle_hello(&mut self, client: &mut Client, args: &[Value]) -> Result<Value, Error> {
        log::info!("hello: ({}) {:?}", client.addrs()[0], args);
        if args.len() == 0 {
            return Error::invalid_args("hello", 0, 1);
        }

        if args[0].as_int() != Some(3) {
            return Error::disconnect(
                "NOPROTO sorry this protocol version is not supported, only RESP3 is supported",
            );
        }

        if args.len() >= 3 {
            let auth = args[1].as_string().unwrap();
            if auth != "auth" || auth != "AUTH" {
                return Error::disconnect("ERR invalid hello command, expected AUTH argument");
            }

            if self.password_required() {
                let username = if args.len() == 3 {
                    "default"
                } else {
                    args[2].as_string().unwrap()
                };
                let password = args[if args.len() == 3 { 2 } else { 3 }]
                    .as_string()
                    .unwrap();
                if !self._check_password(username, password) {
                    return Error::disconnect("ERR invalid password");
                }
            }

            client.authenticated = true;
        } else if self.password_required() && !client.authenticated {
            return Error::disconnect("ERR password required");
        }

        return Ok(map! {
            "server" => "worm",
            "version" => VERSION,
            "proto" => 3,
        });
    }

    fn handle_auth(&mut self, client: &mut Client, args: &[Value]) -> Result<Value, Error> {
        log::info!("auth: ({}) {:?}", client.addrs()[0], args);

        if args.len() == 0 {
            return Error::invalid_args("auth", 0, 1);
        }

        let username = if args.len() == 1 {
            "default"
        } else {
            args[1].as_string().unwrap()
        };

        let password = args[if args.len() == 1 { 0 } else { 1 }]
            .as_string()
            .unwrap();

        if !self._check_password(username, password) {
            return Error::disconnect("ERR invalid password");
        }

        client.authenticated = true;

        return Ok(Value::ok());
    }

    fn handle_commands(&mut self, _client: &mut Client, _args: &[Value]) -> Result<Value, Error> {
        let mut commands: Vec<Value> = self.commands().iter().map(|x| (*x).into()).collect();
        commands.extend_from_slice(&[
            "hello".into(),
            "auth".into(),
            "ping".into(),
            "commands".into(),
        ]);
        Ok(Value::Array(commands))
    }

    fn handle_ping(&mut self, _client: &mut Client, args: &mut Vec<Value>) -> Result<Value, Error> {
        if args.len() > 0 {
            Ok(args[0].clone())
        } else {
            Ok("PONG".into())
        }
    }

    async fn handle(
        handle: Handle<Self>,
        client: &mut Client,
        mut command: Command,
    ) -> Result<Value, Error> {
        log::info!("command: ({}) {:?}", client.addrs()[0], command);
        let mut x = handle.lock();
        if !client.authenticated && !x.password_required() {
            client.authenticated = true
        }

        match command.name() {
            "hello" => return x.handle_hello(client, command.args()),
            "auth" => return x.handle_auth(client, command.args()),
            _ if !client.authenticated => return Error::disconnect("ERR invalid handshake"),
            "commands" => return x.handle_commands(client, command.args()),
            "ping" => return x.handle_ping(client, command.args_mut()),
            _ => (),
        }

        if !client.authenticated {
            return Error::disconnect("ERR unauthorized");
        }

        log::info!("command: ({}) {:?}", client.addrs()[0], command);

        let cmd = x.command(command.name()).map(|cmd| cmd.0.clone());
        if let Some(cmd) = cmd {
            (cmd)(&mut x, client, command)
        } else {
            Ok(Value::error("NOCOMMAND invalid command"))
        }
    }
}

async fn on_command<T: Handler>(data: Handle<T>, client: &mut Client) -> Result<bool, Error> {
    let value = client.read().await?;
    let mut response = true;

    if let Value::Array(mut cmd) = value {
        let name = cmd.remove(0);
        if let Value::String(s) = name {
            let cmd = Command::new(s).with_args(cmd);
            let res = match T::handle(data, client, cmd).await {
                Ok(x) => x,
                Err(Error::Disconnect(e)) => {
                    log::info!("disconnect: ({}) {:?}", client.addrs()[0], e);
                    response = false;
                    Value::Error(e)
                }
                Err(e) => Err(e).into(),
            };
            client.write(&res).await?;
            client.flush().await?;
        }
    }

    Ok(response)
}

impl<T: 'static + Handler + Send> Server<T> {
    pub fn new(data: T) -> Self {
        Server { data }
    }

    pub async fn run<A: tokio::net::ToSocketAddrs>(self, addr: A) -> Result<(), Error> {
        let conn = tokio::net::TcpListener::bind(addr).await?;
        let data = std::sync::Arc::new(std::sync::Mutex::new(self.data));
        loop {
            let (socket, addr) = conn.accept().await?;
            let data = data.clone();
            tokio::spawn(async move {
                let mut client = Client::new_from_stream(socket, vec![addr], None)
                    .await
                    .unwrap();
                loop {
                    match on_command(Handle(data.clone()), &mut client).await {
                        Ok(true) => continue,
                        Ok(false) => {
                            log::info!("disconnecting: {}", client.addrs()[0]);
                            break;
                        }
                        Err(e) => {
                            log::error!("fatal error: {:?}", e);
                            break;
                        }
                    }
                }
            });
        }
    }
}
