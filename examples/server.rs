use worm::*;

#[derive(Default)]
pub struct KV {
    store: std::collections::BTreeMap<String, String>
}

#[async_trait::async_trait]
impl Handler for KV {
    async fn handle(handle: Handle<Self>, _client: &Client, command: Command) ->  Result<Value, Error> {
        match command.name() {
            "set" => {
                handle.lock().store.insert("Test".to_string(), "Value".to_string());
                Ok("OK".into())
            }
            "get" => {
                Ok(handle.lock().store.get("Test").cloned().into())
            }
            _ => Ok(command.into())
        }
    }
}

pub fn main() {
    Server::new(KV::default()).run("127.0.0.1:8080").unwrap();
}
