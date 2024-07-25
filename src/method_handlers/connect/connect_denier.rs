use crate::{protocol::Reply, Socks5Error};

use super::Connect;

/// The `ConnectDenier` struct is an implementation of the `Connect` trait that denies all
/// connection requests. It is used to reject any attempt to establish a TCP connection.
pub struct ConnectDenier;

impl<C> Connect<C> for ConnectDenier
where
    C: Send + Sync,
{
    type ServerConnection = ();

    async fn establish_connection(
        &mut self,
        _: crate::protocol::SocksSocketAddr,
        _: C,
    ) -> crate::Result<Self::ServerConnection> {
        Err(Socks5Error::Socks5Error(Reply::CommandNotSupported))
    }

    async fn start_listening<T>(
        &mut self,
        _: &mut T,
        _: Self::ServerConnection,
    ) -> crate::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        unreachable!()
    }
}
