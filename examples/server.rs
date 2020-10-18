use worm::*;


#[derive(Default, worm::Handler)]
#[commands(get, set, del, list)]
#[password(authorize)]
pub struct KV {
    store: Map,
}

impl KV {
    async fn set(this: Handle<Self>, _client: std::pin::Pin<&mut Client>, mut command: Command) -> Result<Value, Error> {
        let key = command.pop_front();
        let value = command.pop_front();
        this.lock().store.insert(key, value);
        Ok(Value::ok())
    }

    async fn get(this: Handle<Self>, _client: std::pin::Pin<&mut Client>, mut command: Command) -> Result<Value, Error> {
        let args = command.args_mut();
        let key = args.remove(0);
        Ok(this.lock().store.get(&key).cloned().into())
    }

    async fn del(this: Handle<Self>, _client: std::pin::Pin<&mut Client>, command: Command) -> Result<Value, Error> {
        let args = command.args();
        this.lock().store.remove(&args[0]);
        Ok(Value::ok())
    }

    async fn list(this: Handle<Self>, client: std::pin::Pin<&mut Client>, _command: Command) -> Result<Value, Error> {
        let client = client.get_mut();
        let this = this.lock();
        client.output.write_array_header(this.store.len()).await?;
        for k in this.store.keys() {
            client.write(k).await?;
        }
        Value::done()
    }

    fn authorize(&self, user: &str, pass: &str) -> bool {
        user == "test" && pass == "test"
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::init();
    Server::new(KV::default()).run("127.0.0.1:8080").await
}
