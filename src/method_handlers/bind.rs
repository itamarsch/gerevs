use std::net::SocketAddr;

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};

use crate::{protocol::SocksSocketAddr, Socks5Error};

pub mod bind_denier;
pub mod tunnel_bind;

pub trait Bind<C> {
    fn bind(
        &mut self,
        addr: SocksSocketAddr,
        _: &C,
    ) -> impl std::future::Future<Output = crate::Result<TcpListener>> {
        async move {
            TcpListener::bind(&*addr.to_socket_addr().await?)
                .await
                .map_err(|err| crate::Socks5Error::Socks5Error(err.kind().into()))
        }
    }

    fn accept(
        &mut self,
        server: TcpListener,
        _: &C,
    ) -> impl std::future::Future<Output = crate::Result<(TcpStream, SocketAddr)>> + Send {
        async move {
            server
                .accept()
                .await
                .map_err(|err| Socks5Error::Socks5Error(err.kind().into()))
        }
    }

    fn start_listening<T>(
        &mut self,
        server: &mut T,
        client: TcpStream,
        credentials: C,
    ) -> impl std::future::Future<Output = crate::Result<()>> + Send
    where
        T: AsyncWrite + AsyncRead + Send + Unpin;
}
