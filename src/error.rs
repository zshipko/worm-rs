use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Parse int: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Parse float: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("Invalid value: {0:?}")]
    InvalidValue(Value),

    #[error("Invalid byte: {0:?}")]
    InvalidByte(Option<u8>),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Disconnect: {0}")]
    Disconnect(String),
}

impl Error {
    pub fn disconnect(s: impl Into<String>) -> Result<Value, Error> {
        Err(Error::Disconnect(s.into()))
    }

    pub fn invalid_args(cmd: impl AsRef<str>, got: usize, expected: usize) -> Result<Value, Error> {
        Ok(Value::Error(format!(
            "ERR wrong number of arguments for {} command, got {} but expected {}",
            cmd.as_ref(),
            got,
            expected
        )))
    }
}
