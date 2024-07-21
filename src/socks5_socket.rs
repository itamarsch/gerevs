use std::io::{self, ErrorKind};

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
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Authentication failed",
                ));
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

        self.flush().await?;

        Ok(())
    }

    async fn write_auth_method(&mut self, auth_method: AuthMethod) -> io::Result<()> {
        self.write_u8(VERSION).await?;
        self.write_u8(auth_method.to_u8()).await?;
        self.flush().await?;
        Ok(())
    }

    async fn parse_methods(&mut self) -> io::Result<Vec<AuthMethod>> {
        let version = self.read_u8().await?;
        if version != VERSION {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unexpected protocol version",
            ));
        }

        let nmethods = self.read_u8().await?;
        if nmethods < 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "No authentication methods provided",
            ));
        }

        let mut methods = vec![0; nmethods as usize];

        self.read_exact(&mut methods).await?;
        let methods = methods
            .into_iter()
            .map(AuthMethod::from_u8)
            .collect::<Vec<_>>();

        Ok(methods)
    }

    async fn parse_request(&mut self) -> io::Result<Command> {
        let version = self.read_u8().await?;
        if version != VERSION {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unexpected protocol version",
            ));
        }

        let command = self.read_u8().await?;
        let Some(command) = Command::from_u8(command) else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid command value",
            ));
        };

        let reserved = self.read_u8().await?;
        if reserved != RESERVED {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unexpected reserved value, expected 0",
            ));
        }

        Ok(command)
    }

    async fn parse_addr(&mut self) -> io::Result<SocksSocketAddr> {
        SocksSocketAddr::read(self).await
    }
}
