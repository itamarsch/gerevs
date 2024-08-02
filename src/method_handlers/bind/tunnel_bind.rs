use std::io;

use tokio::net::{TcpListener, TcpStream};

use super::Bind;
/// The `TunnelBind` struct is an implementation of the `Bind` trait that handles TCP BIND requests
/// by establishing a TCP connection on a random available port and relaying data between the client and the target server.
///
/// This is a simple and basic implementation that binds to a local address and directly relays
/// TCP packets between the client and the target server without any additional processing or filtering.
///
/// This struct can be used in scenarios where basic TCP traffic needs to be tunneled through
/// a SOCKS5 proxy server without any special handling or configuration.
pub struct TunnelBind;

impl Bind<()> for TunnelBind {
    type Listener = TcpListener;

    type Stream = TcpStream;

    async fn start_listening<'a, T>(
        self,
        mut server: T,
        mut client: tokio::net::TcpStream,
        _: (),
    ) -> crate::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        let res = tokio::io::copy_bidirectional(&mut server, &mut client)
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

    async fn bind(
        &mut self,
        addr: crate::protocol::SocksSocketAddr,
        _: &(),
    ) -> crate::Result<(std::net::SocketAddr, Self::Listener)> {
        let addrs = &*addr.to_socket_addr().await?;
        let listener = TcpListener::bind(addrs).await?;
        let bound_addr = listener.local_addr()?;
        Ok((bound_addr, listener))
    }

    async fn accept(
        &mut self,
        server: Self::Listener,
        _: &(),
    ) -> crate::Result<(Self::Stream, std::net::SocketAddr)> {
        let res = server.accept().await?;
        Ok(res)
    }
}
