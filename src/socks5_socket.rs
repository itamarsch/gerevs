use std::io::{self, ErrorKind};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, instrument};

use crate::auth::Authenticator;

use crate::method_handlers::{Associate, Bind, Connect};
use crate::protocol::{AuthMethod, Command, Reply, SocksSocketAddr, RESERVED, VERSION};

/// The `Sock5Socket` struct represents a SOCKS5 protocol handler that manages the connection
/// between a client and a server. It handles authentication, command parsing, and the execution
/// of the CONNECT, BIND, and UDP ASSOCIATE commands.
pub struct Sock5Socket<T, A, Connect, Bind, Associate> {
    inner: T,
    authenticator: A,
    connect_handler: Connect,
    bind_handler: Bind,
    associate_handler: Associate,
}

mod associate;
mod bind;
mod connect;

impl<T, Auth, C, B, A> Sock5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
{
    /// Creates a new `Sock5Socket` instance.
    ///
    /// - `inner`: The underlying I/O stream.
    /// - `authenticator`: The authenticator for handling client authentication.
    /// - `connect_handler`: The handler for the CONNECT command.
    /// - `bind_handler`: The handler for the BIND command.
    /// - `associate_handler`: The handler for the UDP ASSOCIATE command.
    ///
    /// # Returns
    ///
    /// A new instance of `Sock5Socket`.
    pub fn new(
        inner: T,
        authenticator: Auth,
        connect_handler: C,
        bind_handler: B,
        associate_handler: A,
    ) -> Self {
        Self {
            inner,
            authenticator,
            connect_handler,
            bind_handler,
            associate_handler,
        }
    }

    #[instrument(skip(self))]
    async fn socks_request(&mut self) -> io::Result<(Command, SocksSocketAddr, Auth::Credentials)> {
        let credentials = self.authenticate().await?;

        let command = self.parse_request().await?;
        let addr = self.parse_addr().await?;
        info!("Command: {:?}, dst: {}", command, addr);

        Ok((command, addr, credentials))
    }

    #[instrument(skip(self))]
    async fn authenticate(&mut self) -> io::Result<Auth::Credentials> {
        let methods = self.parse_methods().await?;
        debug!("Received methods: {:?}", methods);

        let method = self.authenticator.select_method(&methods);
        debug!("Selected method: {:?}", method);
        self.write_auth_method(method).await?;

        let credentials = match self
            .authenticator
            .authenticate(&mut self.inner, method)
            .await
        {
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
        debug!("Authentication success");
        Ok(credentials)
    }
}

impl<T, Auth, C, B, A> Sock5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncWrite + AsyncRead + Send + Unpin,
    Auth: Authenticator<T>,
    Auth::Credentials: Sync + Send,
    A: Associate<Auth::Credentials>,
    B: Bind<Auth::Credentials>,
    C: Connect<Auth::Credentials>,
{
    /// Runs the SOCKS5 protocol handler. It handles client requests, including CONNECT,
    /// BIND, and UDP ASSOCIATE commands, and forwards data between the client and the server.
    ///
    /// # Returns
    ///
    /// A future that resolves to `crate::Result<()>` indicating the success or failure of the operation.
    pub async fn run(&mut self) -> crate::Result<()> {
        let (command, addr, credentials) = self.socks_request().await?;
        match command {
            Command::Connect => self.connect(addr, credentials).await?,
            Command::Bind => self.bind(addr, credentials).await?,
            Command::UdpAssociate => self.associate(addr, credentials).await?,
        };
        Ok(())
    }
}

impl<T, Auth, C, B, A> Sock5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub(crate) async fn reply(
        &mut self,
        reply: Reply,
        bnd_address: SocksSocketAddr,
    ) -> io::Result<()> {
        self.inner.write_u8(VERSION).await?;

        self.inner.write_u8(reply.to_u8()).await?;

        self.inner.write_u8(RESERVED).await?;

        self.inner.write_all(&bnd_address.to_bytes()).await?;

        self.inner.flush().await?;

        Ok(())
    }

    async fn write_auth_method(&mut self, auth_method: AuthMethod) -> io::Result<()> {
        self.inner.write_u8(VERSION).await?;
        self.inner.write_u8(auth_method.to_u8()).await?;
        self.inner.flush().await?;
        Ok(())
    }

    async fn parse_methods(&mut self) -> io::Result<Vec<AuthMethod>> {
        let version = self.inner.read_u8().await?;
        if version != VERSION {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unexpected protocol version",
            ));
        }

        let nmethods = self.inner.read_u8().await?;
        if nmethods < 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "No authentication methods provided",
            ));
        }

        let mut methods = vec![0; nmethods as usize];

        self.inner.read_exact(&mut methods).await?;
        let methods = methods
            .into_iter()
            .map(AuthMethod::from_u8)
            .collect::<Vec<_>>();

        Ok(methods)
    }

    async fn parse_request(&mut self) -> io::Result<Command> {
        let version = self.inner.read_u8().await?;
        if version != VERSION {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unexpected protocol version",
            ));
        }

        let command = self.inner.read_u8().await?;
        let Some(command) = Command::from_u8(command) else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid command value",
            ));
        };

        let reserved = self.inner.read_u8().await?;
        if reserved != RESERVED {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unexpected reserved value, expected 0",
            ));
        }

        Ok(command)
    }

    async fn parse_addr(&mut self) -> io::Result<SocksSocketAddr> {
        SocksSocketAddr::read(&mut self.inner).await
    }
}
