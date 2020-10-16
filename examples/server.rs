use worm::*;

#[derive(Default, worm::Handler)]
#[commands(get, set, del)]
pub struct KV {
    store: Map,
}

impl KV {
    fn set(&mut self, _client: &mut Client, command: Command) -> Result<Value, Error> {
        let args = command.args();
        self.store.insert(args[0].clone(), args[1].clone());
        Ok(Value::ok())
    }

    fn get(&mut self, _client: &mut Client, command: Command) -> Result<Value, Error> {
        let args = command.args();
        Ok(self.store.get(&args[0]).cloned().into())
    }


    fn del(&mut self, _client: &mut Client, command: Command) -> Result<Value, Error> {
        let args = command.args();
        self.store.remove(&args[0]);
        Ok(Value::ok())
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    Server::new(KV::default()).run("127.0.0.1:8080").await
}
