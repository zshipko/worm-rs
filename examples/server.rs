use worm::*;

pub struct Echo;


#[async_trait::async_trait]
impl Handler for Echo {
    async fn handle(_handle: Handle<Self>, _client: &Client, command: Command) ->  Result<Value, Error> {
        Ok(Value::Array(command.0))
    }
}

pub fn main() {
    Server::new(Echo).run("127.0.0.1:8080").unwrap();
}
