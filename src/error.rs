#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Parse int: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Parse float: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("Invalid value: {0:?}")]
    InvalidValue(crate::Value),

    #[error("Invalid byte: {0:?}")]
    InvalidByte(Option<u8>),
}
