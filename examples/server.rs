use worm::*;


#[derive(Default, worm::Handler)]
#[commands(get, set, del, list)]
#[password(authorize)]
pub struct KV {
    store: Map,
}

impl KV {
    async fn set(&mut self, _client: std::pin::Pin<&mut Client>, mut command: Command) -> Result<Value, Error> {
        let key = command.pop_front();
        let value = command.pop_front();
        self.store.insert(key, value);
        Ok(Value::ok())
    }

    async fn get(&mut self, client: std::pin::Pin<&mut Client>, mut command: Command) -> Result<Value, Error> {
        let key = command.pop_front();
        if let Some(value) = self.store.get(&key) {
            client.get_mut().write(value).await?;
            return Value::done()
        }

        Ok(Value::Null)
    }

    async fn del(&mut self, _client: std::pin::Pin<&mut Client>, command: Command) -> Result<Value, Error> {
        let args = command.args();
        self.store.remove(&args[0]);
        Ok(Value::ok())
    }

    async fn list(&mut self, client: std::pin::Pin<&mut Client>, _command: Command) -> Result<Value, Error> {
        let client = client.get_mut();
        let this = self;
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
