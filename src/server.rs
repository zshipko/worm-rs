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

impl <'a, T> Commands<'a, T> {
    pub fn new() -> Self {
        Commands(Default::default())
    }

    pub fn add<F: 'static + Fn(&mut T, &mut Client, Command) -> Response>(mut self, key: &'a str, f: F) -> Self {
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

    fn password_required(&self) -> bool {
        false
    }

    fn check_password(&self, _username: &str, _password: &str) -> bool {
        false
    }

    async fn handle(handle: Handle<Self>, client: &mut Client, command: Command) ->  Result<Value, Error> {
        if command.name() == "HELLO" || command.name() == "hello" {
            let args = command.args();
            if args.len() == 1 {
                return Ok(Value::error("NOPROTO sorry this protocol version is not supported"));
            }

            let x = handle.lock();
            if args.len() >= 4 {
                let username = args[2].as_string().unwrap();
                let password = args[3].as_string().unwrap();

                if x.password_required() {
                    if !x.check_password(username, password) {
                        return Error::disconnect("ERR invalid password")
                    }
                }
            } else if x.password_required() {
                return Error::disconnect("ERR password required");
            }

            return Ok(map!{
                "server" => "worm",
                "version" => VERSION,
                "proto" => 3,
            })
        }

        let cmd = handle.lock().command(command.name()).map(|cmd| cmd.0.clone());
        if let Some(cmd) = cmd {
            (cmd)(&mut handle.lock(), client, command)
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
            let (res, disconnect) = match T::handle(data, client, Command(s, cmd)).await {
                Ok(x) => (x, false),
                Err(Error::Disconnect(e)) => (Value::Error(e), true),
                Err(e) => (Err(e).into(), false),
            };
            response = !disconnect;
            client.write(&res).await?;
            client.flush().await?;
        }
    }
    Ok(response)
}

impl<T: 'static + Handler + Send> Server<T> {
    pub fn new(data: T) -> Self {
        Server {
            data,
        }
    }

    pub async fn run<A: tokio::net::ToSocketAddrs>(self, addr: A) -> Result<(), Error> {
        let conn = tokio::net::TcpListener::bind(addr).await?;
        let data = std::sync::Arc::new(std::sync::Mutex::new(self.data));
        loop {
            let (socket, addr) = conn.accept().await?;
            let data = data.clone();
            tokio::spawn(async move {
                let data = data.clone();
                let mut client = Client::new_from_stream(socket, vec![addr], None).await.unwrap();
                loop {
                    match on_command(Handle(data.clone()), &mut client).await {
                        Ok(true) => continue,
                        Ok(false) | Err(_) => break,
                    }
                }
            });
        }
    }
}
