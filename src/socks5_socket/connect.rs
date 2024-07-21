use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    auth::Authenticator,
    method_handlers::Connect,
    protocol::{Reply, SocksSocketAddr},
    Socks5Error,
};

use super::Sock5Socket;
impl<T, Auth, C, B> Sock5Socket<T, Auth, C, B>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
    C: Connect<Auth::Credentials>,
{
    pub async fn connect(
        &mut self,
        addr: SocksSocketAddr,
        credntials: Auth::Credentials,
    ) -> crate::Result<()> {
        let connect_inner = || async {
            let conn = self
                .connect_handler
                .establish_connection(addr.clone(), credntials)
                .await?;

            self.reply(Reply::Success, addr).await?;

            self.connect_handler
                .start_listening(&mut self.inner, conn)
                .await?;

            Ok(())
        };

        let res: crate::Result<_> = connect_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = res {
            self.reply(err, Default::default()).await?;
        }
        Ok(())
    }
}
