use std::io;

use tokio::net::TcpStream;

use crate::protocol::SocksSocketAddr;

use super::Connect;

/// The `TunnelConnect` struct is an implementation of the `Connect` trait that handles
/// TCP CONNECT requests by establishing a TCP connection and relaying data between the client
/// and the target server.
///
/// This is a simple and basic implementation that establishes a direct TCP connection to the target
/// server and relays data between the client and the server without any additional processing or filtering.
///
/// This struct can be used in scenarios where basic TCP traffic needs to be tunneled through
/// a SOCKS5 proxy server without any special handling or configuration.
pub struct TunnelConnect;

impl Connect<()> for TunnelConnect {
    type ServerConnection = TcpStream;

    async fn establish_connection(
        &mut self,
        addr: SocksSocketAddr,
        _credentials: (),
    ) -> crate::Result<TcpStream> {
        let res = TcpStream::connect(&*addr.to_socket_addr().await?).await?;
        Ok(res)
    }

    async fn start_listening<T>(
        &mut self,
        mut client: T,
        mut server: TcpStream,
    ) -> crate::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        let res = tokio::io::copy_bidirectional(&mut client, &mut server)
            .await
            .map(|_| ());
        if let Err(err) = &res {
            if matches!(err.kind(), io::ErrorKind::NotConnected) {
                return Ok(());
            }
        }
        res?;
        Ok(())
    }
}
