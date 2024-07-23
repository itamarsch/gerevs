use std::{
    io::{self},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    select,
};

use crate::{
    auth::Authenticator,
    method_handlers::Associate,
    protocol::{Reply, SocksSocketAddr},
    Socks5Error,
};

use self::udp_message::UdpMessage;

use super::Sock5Socket;

mod udp_message;
async fn addrs_match(client_addrs: &[SocketAddr], udp_addr: &SocketAddr) -> bool {
    for sa in client_addrs.iter() {
        if sa.port() == 0 {
            continue;
        }

        match (sa, udp_addr) {
            (SocketAddr::V4(sa_v4), SocketAddr::V4(udp_v4)) => {
                if (*sa_v4.ip() == Ipv4Addr::UNSPECIFIED || sa_v4.ip() == udp_v4.ip())
                    && sa_v4.port() == udp_v4.port()
                {
                    return true;
                }
            }
            (SocketAddr::V6(sa_v6), SocketAddr::V6(udp_v6)) => {
                if (*sa_v6.ip() == Ipv6Addr::UNSPECIFIED || sa_v6.ip() == udp_v6.ip())
                    && sa_v6.port() == udp_v6.port()
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

impl<T, Auth, C, B, A> Sock5Socket<T, Auth, C, B, A>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
    Auth::Credentials: Sync + Send,
    A: Associate<Auth::Credentials>,
{
    async fn udp_associate_handshake(
        &mut self,
        credentials: &Auth::Credentials,
    ) -> crate::Result<A::Connection> {
        let (peer_host, conn) = self
            .associate_handler
            .bind(credentials)
            .await
            .map_err(|err| Socks5Error::Socks5Error(err.kind().into()))?;

        self.reply(Reply::Success, peer_host.into()).await?;

        Ok(conn)
    }

    async fn udp_listen(
        &mut self,
        mut conn: A::Connection,
        client_addrs: &[SocketAddr],
        credntials: &Auth::Credentials,
    ) -> crate::Result<()> {
        let mut verified_client_addr = None;
        let mut buf = [0; 4096];
        let mut tcp_buf = [0; 1];
        loop {
            let (n, source) = select! {
                result = self.associate_handler.recv_from(&mut conn,&mut buf, credntials) => {
                    let Ok((n, source)) = result else {
                        continue;
                    };

                    (n,source)
                }

                tcp_read = self.inner.read(&mut tcp_buf) => {
                    break match tcp_read {
                        Ok(0) => Ok(()),
                        Err(err) => Err(err.into()),
                        Ok(_) => {
                            Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Received unexpected data from tcpstream",
                            )
                            .into())
                        }
                    };
                }
            };

            if verified_client_addr.is_none() && addrs_match(client_addrs, &source).await {
                verified_client_addr = Some(source);
            }

            // No client, ignore message
            let Some(addr) = verified_client_addr else {
                continue;
            };

            let res = if addr == source {
                self.forward_to_server(&mut conn, &buf[..n], credntials)
                    .await
            } else {
                self.forward_to_client(&mut conn, &buf[..n], source, client_addrs, credntials)
                    .await
            };
            if res.is_err() {
                continue;
            }
        }
    }

    async fn forward_to_server(
        &mut self,
        conn: &mut A::Connection,
        buf: &[u8],
        credntials: &Auth::Credentials,
    ) -> io::Result<usize> {
        let udp_message = UdpMessage::parse(buf).await?;
        let dst = &*udp_message.dst.to_socket_addr().await?;

        self.associate_handler
            .send_to(conn, udp_message.data, dst, credntials)
            .await
    }

    async fn forward_to_client(
        &mut self,
        conn: &mut A::Connection,
        buf: &[u8],
        source: SocketAddr,
        client_addrs: &[SocketAddr],
        credentials: &Auth::Credentials,
    ) -> io::Result<usize> {
        let response = UdpMessage {
            fragment_number: 0,
            dst: source.into(),
            data: buf,
        };
        self.associate_handler
            .send_to(conn, &response.as_bytes()[..], client_addrs, credentials)
            .await
    }

    pub async fn associate(
        &mut self,
        addr: SocksSocketAddr,
        credentials: Auth::Credentials,
    ) -> crate::Result<()> {
        let associate_inner = || async {
            let credentials = credentials;
            let client_addrs = addr;

            let client_addrs = &*client_addrs
                .to_socket_addr()
                .await
                .map_err(Socks5Error::from)?;

            let conn = self.udp_associate_handshake(&credentials).await?;
            self.udp_listen(conn, client_addrs, &credentials).await
        };

        let res: crate::Result<()> = associate_inner().await;
        match &res {
            Err(Socks5Error::Socks5Error(err)) => {
                println!("Error: {:?}", err);
                self.reply(*err, Default::default()).await?;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
            _ => {}
        }
        Ok(())
    }
}
