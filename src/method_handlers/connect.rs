use std::io;

use tokio::io::{AsyncRead, AsyncWrite};
pub mod tunnel_connect;
use crate::protocol::SocksSocketAddr;

pub trait Connect<C> {
    type ServerConnection;
    async fn establish_connection(
        &mut self,
        destination: SocksSocketAddr,
        credentials: C,
    ) -> io::Result<Self::ServerConnection>;

    async fn start_listening<T>(
        &mut self,
        client: &mut T,
        connection: Self::ServerConnection,
    ) -> io::Result<()>
    where
        T: AsyncWrite + AsyncRead + Send + Unpin;
}
