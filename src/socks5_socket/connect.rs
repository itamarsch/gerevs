use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, info, instrument};

use crate::{
    auth::Authenticator,
    method_handlers::Connect,
    protocol::{Reply, SocksSocketAddr},
    Socks5Error,
};

use super::Socks5Socket;
impl<T, Auth, C, B, A> Socks5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    Auth: Authenticator<T>,
    C: Connect<Auth::Credentials>,
{
    #[instrument(skip_all)]
    pub(crate) async fn connect(
        mut self,
        addr: SocksSocketAddr,
        credentials: Auth::Credentials,
    ) -> crate::Result<()> {
        let connect_inner = || async {
            let conn = self
                .connect_handler
                .establish_connection(addr.clone(), credentials)
                .await
                .map_err(|err| Socks5Error::Socks5Error(err.into()))?;

            debug!("Connection established with: {}", addr);
            self.reply(Reply::Success, addr.clone()).await?;

            info!("Connection with {} closed succefully", addr);

            Ok(conn)
        };

        let res: crate::Result<_> = connect_inner().await;
        let conn = match res {
            Err(Socks5Error::Socks5Error(err)) => {
                self.reply(err, Default::default()).await?;
                return Err(Socks5Error::Socks5Error(err));
            }
            Err(err) => {
                return Err(err);
            }
            Ok(conn) => conn,
        };

        self.connect_handler
            .start_listening(self.inner, conn)
            .await?;
        Ok(())
    }
}
