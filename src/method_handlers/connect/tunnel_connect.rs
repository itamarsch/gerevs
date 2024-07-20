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
    ) -> crate::Result<TcpStream> {
        let res = TcpStream::connect(addr.to_socket_addr()?).await?;
        Ok(res)
    }

    async fn start_listening<T>(
        &mut self,
        client: &mut T,
        mut server: TcpStream,
    ) -> crate::Result<()>
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
        res?;
        Ok(())
    }
}
