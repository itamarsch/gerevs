use crate::{protocol::Reply, Socks5Error};

use super::Bind;

pub struct BindDenier;
impl<C> Bind<C> for BindDenier
where
    C: Send + Sync,
{
    async fn bind(
        &mut self,
        _: crate::protocol::SocksSocketAddr,
        _: &C,
    ) -> crate::Result<tokio::net::TcpListener> {
        Err(Socks5Error::Socks5Error(Reply::CommandNotSupported))
    }

    async fn accept(
        &mut self,
        _: tokio::net::TcpListener,
        _: &C,
    ) -> crate::Result<(tokio::net::TcpStream, std::net::SocketAddr)> {
        unreachable!()
    }

    async fn start_listening<T>(
        &mut self,
        _: &mut T,
        _: tokio::net::TcpStream,
        _: C,
    ) -> crate::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        unreachable!()
    }
}
