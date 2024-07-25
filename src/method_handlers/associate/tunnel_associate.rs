use std::net::SocketAddr;

use tokio::net::UdpSocket;

use super::Associate;

/// The `TunnelAssociate` struct is an implementation of the `Associate` trait that handles
/// UDP associate requests by establishing a UDP relay. It binds to a local address and
/// allows sending and receiving UDP packets through the relay.
///
/// This is a simple and basic implementation that binds to a random available port on the local
/// machine and directly relays UDP packets between the client and the target server without
/// any additional processing or filtering.
///
/// This struct can be used in scenarios where basic UDP traffic needs to be tunneled through
/// a SOCKS5 proxy server without any special handling or configuration.pub struct TunnelAssociate;
pub struct TunnelAssociate;

impl<C> Associate<C> for TunnelAssociate
where
    C: Sync + Send,
{
    type Connection = UdpSocket;
    async fn bind(&self, _: &C) -> crate::Result<(SocketAddr, Self::Connection)> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let peer_addr = socket.local_addr()?;
        Ok((peer_addr, socket))
    }

    async fn send_to<A>(
        &mut self,
        conn: &mut Self::Connection,
        buf: &[u8],
        dst: A,
        _: &C,
    ) -> crate::Result<usize>
    where
        A: tokio::net::ToSocketAddrs,
    {
        let res = conn.send_to(buf, dst).await?;
        Ok(res)
    }

    async fn recv_from(
        &mut self,
        conn: &mut Self::Connection,
        buf: &mut [u8],
        _: &C,
    ) -> crate::Result<(usize, std::net::SocketAddr)> {
        let res = conn.recv_from(buf).await?;
        Ok(res)
    }
}
