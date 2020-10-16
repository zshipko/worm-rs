use crate::internal::*;

#[derive(Debug, Default)]
pub struct Command(pub Vec<Value>);

impl Command {
    pub fn new(name: impl AsRef<str>) -> Command {
        Command(vec![name.as_ref().into()])
    }

    pub fn arg(mut self, x: impl Into<String>) -> Command {
        self.0.push(x.into().into());
        self
    }
}

impl From<Command> for Value {
    fn from(cmd: Command) -> Value {
        Value::Array(cmd.0)
    }
}
