use tokio::io::{AsyncRead, AsyncWrite};
pub mod connect_denier;
pub mod tunnel_connect;
use crate::protocol::SocksSocketAddr;

/// The `Connect` trait defines the necessary operations for handling the SOCKS5 CONNECT command.
/// This command is used to establish a TCP connection to a target server through a SOCKS5 proxy server.
/// The trait includes methods for establishing a connection and starting to listen on that connection.
///
/// ## Type Parameters
///
/// - `C`: The type of credentials required for the connect operations.
pub trait Connect<C> {
    /// The type representing a server connection that can be used for TCP connect operations.
    type ServerConnection;

    /// Establishes a TCP connection to the specified destination address using the provided credentials.
    /// It returns a future that resolves to a result containing the server connection object.
    ///
    /// - `destination`: The target address to which the connection should be established.
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<Self::ServerConnection>`.
    fn establish_connection(
        &mut self,
        destination: SocksSocketAddr,
        credentials: C,
    ) -> impl std::future::Future<Output = crate::Result<Self::ServerConnection>> + Send;

    /// Starts listening on the established server connection and forwards data between the client
    /// and the server connection. It returns a future that resolves to a result indicating the
    /// success or failure of the operation.
    ///
    /// - `client`: A mutable reference to the client connection.
    /// - `connection`: The established server connection.
    /// - Returns: A future that resolves to `crate::Result<()>`.
    fn start_listening<T>(
        self,
        client: T,
        connection: Self::ServerConnection,
    ) -> impl std::future::Future<Output = crate::Result<()>> + Send
    where
        T: AsyncWrite + AsyncRead + Send + Unpin;
}
