use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::SocksSocketAddr;

pub mod bind_denier;
pub mod tunnel_bind;

/// The `Bind` trait defines the necessary operations for handling the SOCKS5 BIND command.
/// This command is used to bind to a specific address and port and wait for incoming TCP connections.
/// The trait includes methods for binding to an address, accepting incoming connections, and starting
/// to listen on the server.
///
/// ## Type Parameters
///
/// - `C`: The type of credentials required for the bind operations.
pub trait Bind<C> {
    /// The type representing a listener that can be used for accepting incoming connections.
    type Listener;

    /// The type representing a stream that can be used for communicating with the client.
    type Stream;

    /// Binds to the specified address and prepares to accept incoming TCP connections.
    /// It returns a future that resolves to a result containing the local socket address and a listener object.
    ///
    /// - `addr`: The address to which the bind operation should be performed.
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<(SocketAddr, Self::Listener)>`.
    fn bind(
        &mut self,
        addr: SocksSocketAddr,
        _: &C,
    ) -> impl std::future::Future<Output = crate::Result<(SocketAddr, Self::Listener)>>;

    /// Accepts an incoming TCP connection on the bound address.
    /// It returns a future that resolves to a result containing the stream and the client's socket address.
    ///
    /// - `server`: The listener that is bound to the address.
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<(Self::Stream, SocketAddr)>`.
    fn accept(
        &mut self,
        server: Self::Listener,
        _: &C,
    ) -> impl std::future::Future<Output = crate::Result<(Self::Stream, SocketAddr)>> + Send;

    /// Starts listening on the server and forwards data between the client and the server.
    /// It returns a future that resolves to a result indicating the success or failure of the operation.
    ///
    /// - `server`: A mutable reference to the server connection.
    /// - `client`: The established client connection.
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<()>`.
    fn start_listening<T>(
        self,
        server: T,
        client: Self::Stream,
        credentials: C,
    ) -> impl std::future::Future<Output = crate::Result<()>> + Send
    where
        T: AsyncWrite + AsyncRead + Send + Unpin + 'static;
}
