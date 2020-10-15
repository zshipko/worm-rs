pub(crate) mod internal {
    pub use std::marker::Unpin;

    pub use async_recursion::async_recursion;
    pub use futures_lite::AsyncBufReadExt;
    pub use smol::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

    pub use crate::*;
}

mod decoder;
mod encoder;
mod error;
mod value;

pub use decoder::Decoder;
pub use encoder::Encoder;
pub use error::Error;
pub use value::{Float, Map, Set, Value};

#[cfg(test)]
mod tests;
