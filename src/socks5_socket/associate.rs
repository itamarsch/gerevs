use std::{
    io::{self, Cursor},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite},
    net::UdpSocket,
};

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

async fn addrs_match(socks_addr: &SocksSocketAddr, udp_addr: &SocketAddr) -> bool {
    if socks_addr.port == 0 || udp_addr.port() == 0 {
        return false;
    }

    if let Ok(resolved_socks_addrs) = socks_addr.to_socket_addr().await {
        for sa in resolved_socks_addrs.iter() {
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
    }
    false
}

impl<T, Auth, C, B> Sock5Socket<T, Auth, C, B>
where
    Self: Unpin + Send,
    T: AsyncRead + AsyncWrite + Unpin + Send,
    Auth: Authenticator<T>,
{
    pub async fn associate(
        &mut self,
        addr: SocksSocketAddr,
        credntials: Auth::Credentials,
    ) -> crate::Result<()> {
        let associate_inner = || async {
            let client_socks_addr = addr;

            let client_socks_addr_resolved = &*client_socks_addr
                .to_socket_addr()
                .await
                .map_err(Socks5Error::from)?;

            let udp_listener = UdpSocket::bind("0.0.0.0:0")
                .await
                .map_err(Socks5Error::from)?;

            let peer_host = udp_listener.local_addr().map_err(Socks5Error::from)?;

            self.reply(Reply::Success, peer_host.into()).await?;

            let mut client_addr = None;
            let mut buf = [0; 4096];
            loop {
                let Ok((n, source)) = udp_listener.recv_from(&mut buf).await else {
                    continue;
                };

                if client_addr.is_none() && addrs_match(&client_socks_addr, &source).await {
                    client_addr = Some(source);
                }
                let Some(addr) = client_addr else {
                    continue;
                };
                if addr == source {
                    let Ok(udp_message) = UdpMessage::parse(&buf[..n]).await else {
                        continue;
                    };

                    let Ok(dst) = udp_message.dst.to_socket_addr().await else {
                        continue;
                    };
                    let dst = &*dst;
                    _ = udp_listener.send_to(udp_message.data, dst).await;
                } else {
                    let response = UdpMessage {
                        fragment_number: 0,
                        dst: source.into(),
                        data: &buf[..n],
                    };
                    _ = udp_listener
                        .send_to(&response.as_bytes()[..], client_socks_addr_resolved)
                        .await;
                }
            }
        };

        let res: crate::Result<()> = associate_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = res {
            self.reply(err, Default::default()).await?;
        }
        Ok(())
    }
}
