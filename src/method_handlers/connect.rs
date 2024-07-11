use tokio::io::{AsyncRead, AsyncWrite};
pub mod connect_denier;
pub mod tunnel_connect;
use crate::protocol::SocksSocketAddr;

pub trait Connect<C> {
    type ServerConnection;
    fn establish_connection(
        &mut self,
        destination: SocksSocketAddr,
        credentials: C,
    ) -> impl std::future::Future<Output = crate::Result<Self::ServerConnection>> + Send;

    fn start_listening<T>(
        &mut self,
        client: &mut T,
        connection: Self::ServerConnection,
    ) -> impl std::future::Future<Output = crate::Result<()>> + Send
    where
        T: AsyncWrite + AsyncRead + Send + Unpin;
}
