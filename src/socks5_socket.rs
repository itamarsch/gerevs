use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr},
    pin::Pin,
};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    auth::Authenticator,
    method_handlers::{Bind, Connect},
    Socks5Error,
};

use crate::protocol::{
    Addr, AddressType, AuthMethod, Command, Reply, SocksSocketAddr, RESERVED, VERSION,
};

pub struct Sock5Socket<T, A, Connect, Bind> {
    inner: T,
    authenticator: A,
    connect_handler: Connect,
    bind_handler: Bind,
}

impl<T, Auth, C, B> Sock5Socket<T, Auth, C, B>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
    C: Connect<Auth::Credentials>,
    B: Bind<Auth::Credentials>,
{
    pub fn new(inner: T, authenticator: Auth, connect_handler: C, bind_handler: B) -> Self {
        Self {
            inner,
            authenticator,
            connect_handler,
            bind_handler,
        }
    }

    pub async fn bind(
        &mut self,
        addr: SocksSocketAddr,
        credentials: Auth::Credentials,
    ) -> crate::Result<()> {
        let bind_inner = || async {
            let server = self.bind_handler.bind(addr, &credentials).await?;

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
    pub async fn socks_request(
        &mut self,
    ) -> io::Result<(Command, SocksSocketAddr, Auth::Credentials)> {
        let credentials = self.authenticate().await?;

        let (command, addr_type) = self.parse_request().await?;
        let addr = self.parse_addr(addr_type).await?;

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

    async fn parse_request(&mut self) -> io::Result<(Command, AddressType)> {
        let mut request: [u8; 4] = [0; 4];
        self.read_exact(&mut request).await?;
        assert_eq!(request[0], VERSION);
        assert_eq!(request[2], RESERVED);
        let command = Command::from_u8(request[1]);
        let addr_type = AddressType::from_u8(request[3]);
        Ok((command, addr_type))
    }

    async fn parse_addr(&mut self, address: AddressType) -> io::Result<SocksSocketAddr> {
        let addr = match address {
            AddressType::Ipv4 => {
                let mut addr = [0; 4];
                self.read_exact(&mut addr).await?;
                Addr::Ipv4(Ipv4Addr::from(addr))
            }
            AddressType::DomainName => {
                let len = self.read_u8().await?;
                let mut domain = vec![0; len as usize];
                self.read_exact(&mut domain[..]).await?;
                let domain = String::from_utf8(domain).map_err(|_| io::ErrorKind::InvalidData)?;
                Addr::Domain(domain)
            }
            AddressType::Ipv6 => {
                let mut addr = [0; 16];
                self.read_exact(&mut addr).await?;
                Addr::Ipv6(Ipv6Addr::from(addr))
            }
        };
        let port = self.read_u16().await?;

        Ok(SocksSocketAddr { port, addr })
    }
}

impl<T, Auth, Connect, Bind> AsyncRead for Sock5Socket<T, Auth, Connect, Bind>
where
    Self: Unpin,
    T: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<T, Auth, Connect, Bind> AsyncWrite for Sock5Socket<T, Auth, Connect, Bind>
where
    Self: Unpin,
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
