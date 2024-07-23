use std::net::SocketAddr;

use tokio::net::UdpSocket;

use super::Associate;

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
