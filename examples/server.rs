use worm::*;

#[derive(Default, worm::Handler)]
#[commands(get, set, del, list)]
#[password(authorize)]
pub struct KV {
    store: Map,
}

impl KV {
    fn set(&mut self, _client: &mut Client, mut command: Command) -> Result<Value, Error> {
        let key = command.pop_front();
        let value = command.pop_front();
        self.store.insert(key, value);
        Ok(Value::ok())
    }

    fn get(&mut self, _client: &mut Client, mut command: Command) -> Result<Value, Error> {
        let args = command.args_mut();
        let key = args.remove(0);
        Ok(self.store.get(&key).cloned().into())
    }

    fn del(&mut self, _client: &mut Client, command: Command) -> Result<Value, Error> {
        let args = command.args();
        self.store.remove(&args[0]);
        Ok(Value::ok())
    }

    async fn list(&mut self, client: &mut Client, _command: Command) -> Result<Value, Error> {
        client.output.write_array_header(self.store.len()).await?;
        let mut keys: Vec<Value> = Vec::new();
        for k in self.store.keys() {
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
