use crate::{protocol::Reply, Socks5Error};

use super::Bind;

/// The `BindDenier` struct is an implementation of the `Bind` trait that denies all
/// bind requests. It is used to reject any attempt to bind to a specific address and port
/// for incoming TCP connections.
pub struct BindDenier;

impl<C> Bind<C> for BindDenier
where
    C: Send + Sync,
{
    type Listener = ();
    type Stream = ();

    async fn bind(
        &mut self,
        _: crate::protocol::SocksSocketAddr,
        _: &C,
    ) -> crate::Result<(std::net::SocketAddr, Self::Listener)> {
        Err(Socks5Error::Socks5Error(Reply::CommandNotSupported))
    }

    async fn accept(
        &mut self,
        _: Self::Listener,
        _: &C,
    ) -> crate::Result<(Self::Stream, std::net::SocketAddr)> {
        unreachable!()
    }

    async fn start_listening<T>(&mut self, _: &mut T, _: Self::Stream, _: C) -> crate::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        unreachable!()
    }
}
