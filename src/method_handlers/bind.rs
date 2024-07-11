use std::net::{SocketAddr, SocketAddrV4};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};

use crate::{
    protocol::{Addr, SocksSocketAddr},
    Socks5Error,
};

pub mod tunnel_bind;

pub trait Bind<C> {
    fn bind(
        &mut self,
        addr: SocksSocketAddr,
        _: &C,
    ) -> impl std::future::Future<Output = crate::Result<TcpListener>> + Send {
        async move {
            match addr.addr {
                Addr::Ipv4(addrv4) => TcpListener::bind(SocketAddrV4::new(addrv4, addr.port)).await,
                Addr::Ipv6(addrv6) => {
                    TcpListener::bind(std::net::SocketAddrV6::new(addrv6, addr.port, 0, 0)).await
                }
                Addr::Domain(ref domain) => {
                    let domain = format!("{}:{}", domain, addr.port);
                    TcpListener::bind(domain).await
                }
            }
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
