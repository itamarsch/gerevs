use std::{
    io,
    net::{SocketAddrV4, SocketAddrV6},
};

use tokio::net::TcpStream;

use crate::protocol::{Addr, SocksSocketAddr};

use super::Connect;

pub struct TunnelConnect;

impl Connect<()> for TunnelConnect {
    type ServerConnection = TcpStream;

    async fn establish_connection(
        &mut self,
        addr: SocksSocketAddr,
        _credentials: (),
    ) -> std::io::Result<TcpStream> {
        match addr.addr {
            Addr::Ipv4(addrv4) => TcpStream::connect(SocketAddrV4::new(addrv4, addr.port)).await,
            Addr::Ipv6(addrv6) => {
                TcpStream::connect(SocketAddrV6::new(addrv6, addr.port, 0, 0)).await
            }
            Addr::Domain(ref domain) => {
                let domain = format!("{}:{}", domain, addr.port);
                TcpStream::connect(domain).await
            }
        }
    }

    async fn start_listening<T>(&mut self, client: &mut T, mut server: TcpStream) -> io::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        let res = tokio::io::copy_bidirectional(client, &mut server)
            .await
            .map(|_| ());
        if let Err(err) = &res {
            if matches!(err.kind(), io::ErrorKind::NotConnected) {
                return Ok(());
            }
        }
        res
    }
}
