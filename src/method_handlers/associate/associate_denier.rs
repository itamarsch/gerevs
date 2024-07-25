use std::net::SocketAddr;

use crate::protocol::Reply;

use super::Associate;

/// The `AssociateDenier` struct is an implementation of the `Associate` trait that denies all
/// UDP associate requests. It is used to reject any attempt to establish a UDP relay.
pub struct AssociateDenier;

impl<C> Associate<C> for AssociateDenier
where
    C: Sync + Send,
{
    type Connection = ();
    async fn bind(&self, _: &C) -> crate::Result<(SocketAddr, Self::Connection)> {
        Err(crate::Socks5Error::Socks5Error(Reply::CommandNotSupported))
    }

    async fn send_to<A>(
        &mut self,
        _: &mut Self::Connection,
        _: &[u8],
        _: A,
        _: &C,
    ) -> crate::Result<usize>
    where
        A: tokio::net::ToSocketAddrs,
    {
        unreachable!()
    }

    async fn recv_from(
        &mut self,
        _: &mut Self::Connection,
        _: &mut [u8],
        _: &C,
    ) -> crate::Result<(usize, std::net::SocketAddr)> {
        unreachable!()
    }
}
