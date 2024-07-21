use std::io;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::auth::Authenticator;

use crate::protocol::{AuthMethod, Command, Reply, SocksSocketAddr, RESERVED, VERSION};

pub struct Sock5Socket<T, A, Connect, Bind> {
    inner: T,
    authenticator: A,
    connect_handler: Connect,
    bind_handler: Bind,
}

pub mod associate;
pub mod bind;
pub mod connect;
pub mod socks5_io;

impl<T, Auth, C, B> Sock5Socket<T, Auth, C, B>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
{
    pub fn new(inner: T, authenticator: Auth, connect_handler: C, bind_handler: B) -> Self {
        Self {
            inner,
            authenticator,
            connect_handler,
            bind_handler,
        }
    }

    pub async fn socks_request(
        &mut self,
    ) -> io::Result<(Command, SocksSocketAddr, Auth::Credentials)> {
        let credentials = self.authenticate().await?;

        let command = self.parse_request().await?;
        let addr = self.parse_addr().await?;

        Ok((command, addr, credentials))
    }

    async fn authenticate(&mut self) -> io::Result<Auth::Credentials> {
        let methods = self.parse_methods().await?;

        let method = self.authenticator.select_method(&methods);
        self.write_auth_method(method).await?;

        let credentials = match self.authenticator.authenticate(&mut self.inner).await {
            Ok(Some(credentials)) => credentials,

            Ok(None) => {
                return Err(io::ErrorKind::InvalidInput.into());
            }
            Err(err) => {
                return Err(err);
            }
        };
        Ok(credentials)
    }
}

impl<T, Auth, C, B> Sock5Socket<T, Auth, C, B>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub async fn reply(&mut self, reply: Reply, bnd_address: SocksSocketAddr) -> io::Result<()> {
        self.write_u8(VERSION).await?;

        self.write_u8(reply.to_u8()).await?;

        self.write_u8(RESERVED).await?;

        self.write_all(&bnd_address.to_bytes()).await?;

        Ok(())
    }

    async fn write_auth_method(&mut self, auth_method: AuthMethod) -> io::Result<()> {
        self.write_u8(VERSION).await?;
        self.write_u8(auth_method.to_u8()).await?;
        Ok(())
    }

    async fn parse_methods(&mut self) -> io::Result<Vec<AuthMethod>> {
        let mut header: [u8; 2] = [0; 2];
        self.read_exact(&mut header).await?;

        assert_eq!(header[0], VERSION);

        let mut methods = vec![0; header[1] as usize];

        self.read_exact(&mut methods).await?;
        let methods = methods
            .into_iter()
            .map(AuthMethod::from_u8)
            .collect::<Vec<_>>();

        Ok(methods)
    }

    async fn parse_request(&mut self) -> io::Result<Command> {
        let mut request: [u8; 3] = [0; 3];
        self.read_exact(&mut request).await?;
        assert_eq!(request[0], VERSION);
        assert_eq!(request[2], RESERVED);
        let command = Command::from_u8(request[1]);
        Ok(command)
    }

    async fn parse_addr(&mut self) -> io::Result<SocksSocketAddr> {
        SocksSocketAddr::read(self).await
    }
}
