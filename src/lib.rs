pub(crate) mod internal {
    pub use std::marker::Unpin;

    pub use async_recursion::async_recursion;
    pub use futures_lite::AsyncBufReadExt;
    pub use smol::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

    pub use smol::prelude::*;

    pub use crate::*;
}

mod client;
mod command;
mod decoder;
mod encoder;
mod error;
mod server;
mod value;

pub use client::Client;
pub use command::Command;
pub use decoder::Decoder;
pub use encoder::Encoder;
pub use error::Error;
pub use server::{Handle, Handler, Server};
pub use value::{Float, Map, Set, Value};

#[cfg(test)]
mod tests;
