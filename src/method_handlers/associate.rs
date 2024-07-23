use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;

pub mod associate_denier;
pub mod tunnel_associate;

pub trait Associate<C>
where
    C: Sync + Send,
{
    type Connection;
    fn bind(
        &self,
        credentials: &C,
    ) -> impl std::future::Future<Output = crate::Result<(SocketAddr, Self::Connection)>> + Send;

    fn send_to<A>(
        &mut self,
        conn: &mut Self::Connection,
        buf: &[u8],
        dst: A,
        credentials: &C,
    ) -> impl std::future::Future<Output = crate::Result<usize>> + Send
    where
        A: ToSocketAddrs + Send;

    fn recv_from(
        &mut self,
        conn: &mut Self::Connection,
        buf: &mut [u8],
        credentials: &C,
    ) -> impl std::future::Future<Output = crate::Result<(usize, SocketAddr)>> + Send;
}
