use tokio::io::{AsyncRead, AsyncWrite};

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
    pub async fn bind(
        &mut self,
        addr: SocksSocketAddr,
        credentials: Auth::Credentials,
    ) -> crate::Result<()> {
        let bind_inner = || async {
            let server = self
                .bind_handler
                .bind(addr, &credentials)
                .await
                .map_err(|err| Socks5Error::Socks5Error(err.into()))?;

            let localaddr = server
                .local_addr()
                .map_err(|err| crate::Socks5Error::Socks5Error(err.kind().into()))?;

            self.reply(Reply::Success, localaddr.into()).await?;

            let (client, client_addr) = self.bind_handler.accept(server, &credentials).await?;

            self.reply(Reply::Success, client_addr.into()).await?;

            self.bind_handler
                .start_listening(&mut self.inner, client, credentials)
                .await?;
            Ok(())
        };

        let res: crate::Result<_> = bind_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = res {
            self.reply(err, Default::default()).await?;
        }
        Ok(())
    }
}
