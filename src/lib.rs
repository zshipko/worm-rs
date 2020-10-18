pub(crate) mod internal {
    pub use std::marker::Unpin;

    pub use async_recursion::async_recursion;
    pub use futures::io::AsyncBufReadExt;
    pub use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

    pub use tokio::prelude::*;

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
pub use server::{Handle, Handler, Response, Server};
pub use value::{Float, Map, Set, Value};

pub use worm_derive::Handler;

pub use async_trait::async_trait;

#[cfg(test)]
mod tests;
