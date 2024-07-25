use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;

pub mod associate_denier;
pub mod tunnel_associate;

/// The `Associate` trait defines the necessary operations for handling the SOCKS5 UDP ASSOCIATE command.
/// This command is used to establish a UDP relay, allowing clients to send and receive UDP packets
/// through a SOCKS5 proxy server. The trait includes methods for binding to a local address, sending
/// UDP packets to a destination, and receiving UDP packets from a source.
///
/// ## Type Parameters
///
/// - `C`: The type of credentials required for the associate operations. It must implement `Sync` and `Send`.
pub trait Associate<C>
where
    C: Sync + Send,
{
    /// The type representing a connection that can be used for UDP associate operations.
    type Connection;

    /// Binds to a local address and prepares to receive UDP packets. It returns a future that
    /// resolves to a result containing the local socket addressa (Needed for the socks5 protocol) and a connection object.
    ///
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<(SocketAddr, Self::Connection)>`.
    fn bind(
        &self,
        credentials: &C,
    ) -> impl std::future::Future<Output = crate::Result<(SocketAddr, Self::Connection)>> + Send;

    /// Sends a UDP packet to the specified destination address. It returns a future that resolves
    /// to a result containing the number of bytes sent.
    ///
    /// - `conn`: A mutable reference to the connection object.
    /// - `buf`: The buffer containing the data to be sent.
    /// - `dst`: The destination address to which the data should be sent.
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<usize>`.
    fn send_to<A>(
        &mut self,
        conn: &mut Self::Connection,
        buf: &[u8],
        dst: A,
        credentials: &C,
    ) -> impl std::future::Future<Output = crate::Result<usize>> + Send
    where
        A: ToSocketAddrs + Send;

    /// Receives a UDP packet from a source address. It returns a future that resolves to a result
    /// containing the number of bytes received and the source address.
    ///
    /// - `conn`: A mutable reference to the connection object.
    /// - `buf`: The buffer to store the received data.
    /// - `credentials`: The credentials required for the operation.
    /// - Returns: A future that resolves to `crate::Result<(usize, SocketAddr)>`.
    fn recv_from(
        &mut self,
        conn: &mut Self::Connection,
        buf: &mut [u8],
        credentials: &C,
    ) -> impl std::future::Future<Output = crate::Result<(usize, SocketAddr)>> + Send;
}
