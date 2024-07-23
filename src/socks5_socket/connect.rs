use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{info, instrument, trace};

use crate::{
    auth::Authenticator,
    method_handlers::Connect,
    protocol::{Reply, SocksSocketAddr},
    Socks5Error,
};

use super::Sock5Socket;
impl<T, Auth, C, B, A> Sock5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
    C: Connect<Auth::Credentials>,
{
    #[instrument(skip_all)]
    pub async fn connect(
        &mut self,
        addr: SocksSocketAddr,
        credntials: Auth::Credentials,
    ) -> crate::Result<()> {
        let connect_inner = || async {
            let conn = self
                .connect_handler
                .establish_connection(addr.clone(), credntials)
                .await
                .map_err(|err| Socks5Error::Socks5Error(err.into()))?;

            trace!("Connection established with: {}", addr);
            self.reply(Reply::Success, addr.clone()).await?;

            self.connect_handler
                .start_listening(&mut self.inner, conn)
                .await?;

            info!("Connection with {} closed succefully", addr);

            Ok(())
        };

        let res: crate::Result<_> = connect_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = &res {
            self.reply(*err, Default::default()).await?;
        }
        res
    }
}
