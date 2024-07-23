use std::{io, net::SocketAddr};

use tokio::net::UdpSocket;

use super::Associate;

pub struct AssociateTunnel;

impl<C> Associate<C> for AssociateTunnel
where
    C: Sync + Send,
{
    type Connection = UdpSocket;
    async fn bind(&self, _: &C) -> io::Result<(SocketAddr, Self::Connection)> {
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
    ) -> std::io::Result<usize>
    where
        A: tokio::net::ToSocketAddrs,
    {
        UdpSocket::send_to(conn, buf, dst).await
    }

    async fn recv_from(
        &mut self,
        conn: &mut Self::Connection,
        buf: &mut [u8],
        _: &C,
    ) -> std::io::Result<(usize, std::net::SocketAddr)> {
        UdpSocket::recv_from(conn, buf).await
    }
}
