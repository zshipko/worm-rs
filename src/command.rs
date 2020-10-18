use crate::internal::*;

#[derive(Debug, Default)]
pub struct Command(pub String, pub Vec<Value>);

impl Command {
    pub fn new(name: impl AsRef<str>) -> Command {
        Command(name.as_ref().to_ascii_lowercase(), vec![])
    }

    pub fn with_args(mut self, x: impl Into<Vec<Value>>) -> Command {
        self.1 = x.into();
        self
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

    pub fn into_vec(mut self) -> Vec<Value> {
        let x = self.0.into();
        self.1.insert(0, x);
        self.1
    }

    pub fn split(self) -> (String, Vec<Value>) {
        (self.0, self.1)
    }

    pub fn pop_front(&mut self) -> Value {
        if self.1.len() == 0 {
            Value::Null
        } else {
            self.1.remove(0)
        }
    }
}

impl From<Command> for Value {
    fn from(mut cmd: Command) -> Value {
        cmd.1.insert(0, cmd.0.into());
        Value::Array(cmd.1)
    }
}
