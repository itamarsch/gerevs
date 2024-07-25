//! # Gerves
//!
//! This crate provides utilities for creating SOCKS5 proxy servers.
//! It includes features like easy integration, asynchronous support,
//! and full SOCKS5 protocol support.
//!
//! ## Overview of the SOCKS5 Protocol
//!
//! SOCKS5 is a protocol that facilitates routing network packets between a client and server through a proxy server. It provides a secure and flexible means of network communication, supporting a variety of authentication methods and addressing schemes.
//!
//! ### Key Features of SOCKS5
//!
//! - **Authentication**: SOCKS5 supports multiple authentication methods, including no authentication, username/password, and GSSAPI-based authentication (Not yet implemented). The protocol allows defining authentication methods of your own The protocol allows defining authentication methods of your own
//! - **Address Types**: It supports IPv4, IPv6, and domain name addressing, making it versatile for different network configurations.
//! - **Connection Types**: SOCKS5 can handle TCP connections and UDP packets, allowing both connection-oriented and connectionless protocols to be proxied.
//!
//! ### Protocol Flow
//!
//! The SOCKS5 protocol involves the following steps:
//!
//! 1. **Handshake**: The client sends a handshake request specifying the authentication methods it supports. The server responds with the authentication method to be used.
//! 2. **Authentication**: If an authentication method other than "no authentication" is selected, the client and server perform the authentication process.
//! 3. **Request**: The client sends a connection request specifying the destination address and port. The server processes this request and establishes a connection to the target server.
//! 4. **Data Transfer**: Once the connection is established, data can be sent and received between the client and the target server through the proxy.
//! 5. **Termination**: The connection is terminated when the client or server closes the connection.
//!
//! ## SOCKS5 Commands
//!
//! The SOCKS5 protocol supports three different commands that a client can issue:
//!
//! 1. **CONNECT**: This command is used to establish a TCP connection to a target server. It is typically used for protocols like HTTP, where a continuous connection is required between the client and the server.
//! 2. **BIND**: This command is used to establish a TCP connection where the client is expecting to receive connections from the target server. It is useful for protocols that require the server to connect back to the client, such as FTP.
//! 3. **UDP ASSOCIATE**: This command is used to establish a UDP relay connection. It allows the client to send and receive UDP packets through the proxy server, which is useful for applications like DNS queries or streaming services that rely on UDP.
//!
//! ### Command Flow
//!
//! - **CONNECT**: The client sends a request to the server with the target address and port. The server establishes the connection to the target server and relays data between the client and the target.
//! - **BIND**: The client sends a request to the server indicating that it wants to bind to a specific address and port. The server then waits for incoming connections from the target server to this address and port.
//! - **UDP ASSOCIATE**: The client sends a request to the server to establish a UDP relay. The server provides the client with an IP and port to send UDP packets to, which the server then relays to the target server.
//!
//! ## Examples
//!
//! Basic usage:
//!
//! ```rust
//! use gerevs::{
//!     auth::NoAuthAuthenticator,
//!     method_handlers::{TunnelAssociate, TunnelBind, TunnelConnect},
//!     Sock5Socket,
//! };
//! use std::error::Error;
//! use tokio::net::{TcpListener, TcpStream};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let server = TcpListener::bind("0.0.0.0:1080").await?;
//!     loop {
//!         let (client, _addr) = server.accept().await?;
//!
//!         tokio::spawn(async move {
//!             let result = handle_connection(client).await;
//!             if let Err(err) = result {
//!                 eprintln!("Failed: {:?}", err);
//!             }
//!         });
//!     }
//! }
//!
//! async fn handle_connection(client: TcpStream) -> gerevs::Result<()> {
//!     let mut sock5_stream = Sock5Socket::new(
//!         client,
//!         NoAuthAuthenticator,
//!         TunnelConnect,
//!         TunnelBind,
//!         TunnelAssociate,
//!     );
//!     sock5_stream.run().await
//! }
//! ```
//!
//! This example demonstrates setting up a basic SOCKS5 proxy server that listens for incoming connections on port 1080 and handles them asynchronously.
//!
//! ### Explanation of Non-Obvious Parts
//!
//! 1. **`NoAuthAuthenticator`**:
//!     - This is a struct from the `gerevs::auth` module.
//!     - It is used to handle the "no authentication" method in the SOCKS5 protocol, meaning no credentials are required for clients to connect.
//!     - This is an implementation of the `auth::Authenticator` trait that selects the `NoAuthRequired` method and performs no sub-negotiation.
//!
//! 2. **`TunnelConnect`, `TunnelBind`, `TunnelAssociate`**:
//!     - These are the most basic implementations of the traits: `method_handlers::Connect`, `method_handlers::Bind`, and `method_handlers::Associate`.
//!     - `TunnelConnect` implements the `Connect` trait, establishing a direct TCP connection to a specified target server.
//!     - `TunnelBind` implements the `Bind` trait, setting up a TCP listener that waits for incoming connections from a target server, and forwards any messages between the two.
//!     - `TunnelAssociate` implements the `Associate` trait, Forwards UDP packets between the client and the target server.
//!
//! 3. **`Sock5Socket`**:
//!     - This is the main struct from the `gerevs` crate that represents a SOCKS5 connection.
//!     - It takes the client TCP stream and the necessary handlers (authenticator and method handlers) to manage the SOCKS5 protocol interactions.
//!
//! 4. **`sock5_stream.run().await`**:
//!     - This method starts the SOCKS5 protocol operations on the given connection.
//!     - It processes the handshake, authentication (if any), and the command handling (CONNECT, BIND, or UDP ASSOCIATE) based on the client's requests.
//!
//! 5. **`gerevs::Result`**:
//!     - A custom result type provided by the `gerevs` crate
//!
//! By understanding these parts, you can see how the `gerevs` crate simplifies the implementation of a SOCKS5 proxy server, handling the complex protocol details and allowing you to focus on the server logic.

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
