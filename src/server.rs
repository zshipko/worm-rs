use crate::internal::*;

pub struct Handle<T: Sync + Send + Sized>(std::sync::Arc<std::sync::Mutex<std::sync::Arc<T>>>);

unsafe impl<T: Sync + Send> Send for Handle<T> {}
unsafe impl<T: Sync + Send> Sync for Handle<T> {}

pub struct Server<T: Sync + Send> {
    data: T,
}

#[async_trait::async_trait]
pub trait Handler: Sized + Sync + Send {
    async fn handle(handle: Handle<Self>, client: &Client, command: Command) ->  Result<Value, Error>;
}

async fn on_command<T: Handler + Sync + Send>(data: Handle<T>, client: &mut Client) -> Result<(), Error> {
    let value = client.read().await?;
    println!("{:?}", value);
    if let Value::Array(cmd) = value {
        let res = T::handle(data, client, Command(cmd)).await?;
        client.write(&res).await?;
        client.flush().await?;
    }
    Ok(())
}

impl<T: 'static + Handler + Sync + Send> Server<T> {
    pub fn new(data: T) -> Self {
        Server {
            data,
        }
    }

    pub fn run<A: smol::net::AsyncToSocketAddrs>(self, addr: A) -> Result<(), Error> {
        smol::block_on(async {
            let conn = smol::net::TcpListener::bind(addr).await?;
            let data = std::sync::Arc::new(std::sync::Mutex::new(std::sync::Arc::new(self.data)));
            let mut incoming = conn.incoming();
            while let Some(stream) = incoming.next().await {
                let stream = stream?;
                let data = data.clone();
                smol::spawn(async move {
                    let data = data.clone();
                    let mut client = Client::new_from_stream(stream, vec![], None).await.unwrap();
                    loop {
                        if let Err(_) = on_command(Handle(data.clone()), &mut client).await {
                            break
                        }
                    }
                }).detach();
            }
            let r: Result<(), Error> = Ok(());
            r
        })?;

        Ok(())
    }
}
