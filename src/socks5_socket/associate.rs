use std::{
    io::{self, Cursor},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite},
    net::UdpSocket,
    select,
};

#[derive(Debug)]
struct UdpMessage<'a> {
    fragment_number: u8,
    dst: SocksSocketAddr,
    data: &'a [u8],
}
impl<'a> UdpMessage<'a> {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(self.data.len() + 32);
        res.extend_from_slice(&RESERVED_16.to_be_bytes());
        res.push(self.fragment_number);
        res.extend(self.dst.to_bytes());
        res.extend_from_slice(self.data);
        res
    }

    async fn parse(buf: &'a [u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(buf);

        let reserved = cursor.read_u16().await?;
        if reserved != RESERVED_16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Reserved bytes need to be zero but weren't",
            ));
        }

        let fragment_number = cursor.read_u8().await?;
        let dst = SocksSocketAddr::read(&mut cursor).await?;

        let current_pos = cursor.stream_position().await? as usize;
        let data = &buf[current_pos..];
        Ok(UdpMessage {
            fragment_number,
            dst,
            data,
        })
    }
}

use crate::{
    auth::Authenticator,
    protocol::{Reply, SocksSocketAddr, RESERVED_16},
    Socks5Error,
};

use super::Sock5Socket;

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

impl<T, Auth, C, B> Sock5Socket<T, Auth, C, B>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
{
    async fn udp_associate_handshake(&mut self) -> crate::Result<UdpSocket> {
        let udp_listener = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(Socks5Error::from)?;

        let peer_host = udp_listener.local_addr().map_err(Socks5Error::from)?;

        self.reply(Reply::Success, peer_host.into()).await?;

        Ok(udp_listener)
    }

    async fn udp_listen(
        &mut self,
        udp_socket: UdpSocket,
        client_addrs: &[SocketAddr],
    ) -> crate::Result<()> {
        let mut verified_client_addr = None;
        let mut buf = [0; 4096];
        let mut tcp_buf = [0; 1];
        loop {
            let (n, source) = select! {
                result = udp_socket.recv_from(&mut buf) => {

                    let Ok((n, source)) = result else {
                        continue;
                    };

                    (n,source)
                }

                tcp_read = self.read(&mut tcp_buf) => {
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
                Self::forward_to_server(&udp_socket, &buf[..n]).await
            } else {
                Self::forward_to_client(&udp_socket, &buf[..n], source, client_addrs).await
            };
            if res.is_err() {
                continue;
            }
        }
    }

    async fn forward_to_server(udp_socket: &UdpSocket, buf: &[u8]) -> io::Result<usize> {
        let udp_message = UdpMessage::parse(buf).await?;
        let dst = &*udp_message.dst.to_socket_addr().await?;

        udp_socket.send_to(udp_message.data, dst).await
    }

    async fn forward_to_client(
        udp_socket: &UdpSocket,
        buf: &[u8],
        source: SocketAddr,
        client_addrs: &[SocketAddr],
    ) -> io::Result<usize> {
        let response = UdpMessage {
            fragment_number: 0,
            dst: source.into(),
            data: buf,
        };
        udp_socket
            .send_to(&response.as_bytes()[..], client_addrs)
            .await
    }

    pub async fn associate(
        &mut self,
        addr: SocksSocketAddr,
        credntials: Auth::Credentials,
    ) -> crate::Result<()> {
        let associate_inner = || async {
            let client_addrs = addr;

            let client_addrs = &*client_addrs
                .to_socket_addr()
                .await
                .map_err(Socks5Error::from)?;

            let udp_socket = self.udp_associate_handshake().await?;
            self.udp_listen(udp_socket, client_addrs).await
        };

        let res: crate::Result<()> = associate_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = res {
            self.reply(err, Default::default()).await?;
        }
        Ok(())
    }
}
