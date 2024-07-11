use std::io;

use protocol::Reply;

pub mod auth;
pub mod method_handlers;
pub mod protocol;
pub mod socks5_socket;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Socks5Error>;

#[derive(Error, Debug)]
pub enum Socks5Error {
    #[error("Socks error")]
    Socks5Error(#[from] Reply),
    #[error("Error in network operation")]
    IoError(#[from] io::Error),
}
