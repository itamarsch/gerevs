use std::{io, net::SocketAddr};
use tokio::net::ToSocketAddrs;

pub mod udp_socket;

pub trait Associate<C> {
    type Connection;
    async fn bind(&self, credentials: &C) -> io::Result<(SocketAddr, Self::Connection)>;

    async fn send_to<A>(
        &mut self,
        conn: &mut Self::Connection,
        buf: &[u8],
        dst: A,
        credentials: &C,
    ) -> io::Result<usize>
    where
        A: ToSocketAddrs;

    async fn recv_from(
        &mut self,
        conn: &mut Self::Connection,
        buf: &mut [u8],
        credentials: &C,
    ) -> io::Result<(usize, SocketAddr)>;
}
