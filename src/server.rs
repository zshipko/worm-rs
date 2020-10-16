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

pub struct Func<T>(std::sync::Arc<dyn Fn(&mut T, &mut Client, Command) -> Result<Value, Error>>);

pub struct Commands<'a, T>(std::collections::BTreeMap<&'a str, Func<T>>);

#[macro_export]
macro_rules! commands {
    ($($x:ident),*$(,)?) => {
        $crate::Commands::new()
        $(
            .add(stringify!($x), Self::$x)
        )*
    }
}

impl <'a, T> Commands<'a, T> {
    pub fn new() -> Self {
        Commands(Default::default())
    }

    pub fn add<F: 'static + Fn(&mut T, &mut Client, Command) -> Result<Value, Error>>(mut self, key: &'a str, f: F) -> Self {
        self.0.insert(key, Func(std::sync::Arc::new(f)));
        self
    }

    pub fn get(&self, name: &str) -> Option<&Func<T>> {
        self.0.get(name)
    }
}

#[async_trait::async_trait]
pub trait Handler: Send + Sized {
    fn commands(&self) -> Commands<Self>;

    async fn handle(handle: Handle<Self>, client: &mut Client, command: Command) ->  Result<Value, Error> {
        let cmd = handle.lock().commands().get(command.name()).map(|cmd| cmd.0.clone());
        if let Some(cmd) = cmd {
            (cmd)(&mut handle.lock(), client, command)
        } else {
            Ok(Value::error("NOCOMMAND invalid command"))
        }
    }
}

async fn on_command<T: Handler>(data: Handle<T>, client: &mut Client) -> Result<(), Error> {
    let value = client.read().await?;
    if let Value::Array(mut cmd) = value {
        let name = cmd.remove(0);
        if let Value::String(s) = name {
            let res = T::handle(data, client, Command(s, cmd)).await?;
            client.write(&res).await?;
            client.flush().await?;
        }
    }
    Ok(())
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
                    if let Err(_) = on_command(Handle(data.clone()), &mut client).await {
                        break
                    }
                }
            });
        }
    }
}
