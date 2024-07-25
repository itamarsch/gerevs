use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, info, instrument};

use crate::{
    auth::Authenticator,
    method_handlers::Bind,
    protocol::{Reply, SocksSocketAddr},
    Socks5Error,
};

use super::Sock5Socket;

impl<T, Auth, C, B, A> Sock5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
    B: Bind<Auth::Credentials>,
{
    #[instrument(skip_all)]
    pub(crate) async fn bind(
        &mut self,
        addr: SocksSocketAddr,
        credentials: Auth::Credentials,
    ) -> crate::Result<()> {
        let bind_inner = || async {
            let (localaddr, server) = self
                .bind_handler
                .bind(addr, &credentials)
                .await
                .map_err(|err| Socks5Error::Socks5Error(err.into()))?;

            debug!("Listening on {}", localaddr);

            self.reply(Reply::Success, localaddr.into()).await?;

            let (client, client_addr) = self
                .bind_handler
                .accept(server, &credentials)
                .await
                .map_err(|err| crate::Socks5Error::Socks5Error(err.into()))?;

            debug!("Accepted client {}, starting to listen", client_addr);

            self.reply(Reply::Success, client_addr.into()).await?;

            self.bind_handler
                .start_listening(&mut self.inner, client, credentials)
                .await?;

            info!("Connection closed");
            Ok(())
        };

        let res: crate::Result<_> = bind_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = &res {
            self.reply(*err, Default::default()).await?;
        }
        res
    }
}
