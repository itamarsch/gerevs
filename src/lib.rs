use std::io;

use protocol::Reply;

pub mod auth;
pub mod method_handlers;
pub(crate) mod protocol;
mod socks5_socket;
pub use socks5_socket::Sock5Socket;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Socks5Error>;

#[derive(Error, Debug)]
pub enum Socks5Error {
    #[error("Socks error")]
    Socks5Error(#[from] Reply),
    #[error("Error in network operation")]
    IoError(#[from] io::Error),
}

impl From<Socks5Error> for Reply {
    fn from(value: Socks5Error) -> Self {
        match value {
            Socks5Error::Socks5Error(r) => r,
            Socks5Error::IoError(io) => io.kind().into(),
        }
    }
}
