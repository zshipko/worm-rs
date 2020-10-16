use crate::internal::*;

#[derive(Debug, Default)]
pub struct Command(pub String, pub Vec<Value>);

impl Command {
    pub fn new(name: impl Into<String>) -> Command {
        Command(name.into(), vec![])
    }

    pub fn arg(mut self, x: impl Into<String>) -> Command {
        self.1.push(x.into().into());
        self
    }

    pub fn name(&self) -> &str {
        self.0.as_str()
    }

    pub fn args(&self) -> &[Value] {
        &self.1
    }

    pub fn args_mut(&mut self) -> &mut Vec<Value> {
        &mut self.1
    }
}

impl From<Command> for Value {
    fn from(mut cmd: Command) -> Value {
        cmd.1.insert(0, cmd.0.into());
        Value::Array(cmd.1)
    }
}
