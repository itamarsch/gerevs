use std::io::Cursor;

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite, BufReader},
    net::UdpSocket,
};

use crate::{
    auth::Authenticator,
    protocol::{Reply, SocksSocketAddr},
    Socks5Error,
};

use super::Sock5Socket;

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
            let listen_addr = addr;

            let udp_listener = UdpSocket::bind("0.0.0.0:0").await?;
            let peer_host = udp_listener.local_addr()?;
            self.reply(Reply::Success, peer_host.into()).await?;

            let mut buf = [0; 4096];
            let (n, source) = udp_listener.recv_from(&mut buf).await?;

            let mut cursor = BufReader::new(Cursor::new(&buf[..n]));
            let reserved = cursor.read_u16().await?;
            let fragment = cursor.read_u8().await?;
            let addr = SocksSocketAddr::read(&mut cursor).await?;

            println!("{:?}, {:?}, {:?}", reserved, fragment, addr);

            let current_pos = cursor.stream_position().await? as usize;
            let data = &buf[current_pos..n];
            println!("{}", String::from_utf8(data.to_owned()).unwrap());

            udp_listener
                .send_to(data, &*addr.to_socket_addr().await?)
                .await?;

            let (n, res_addr) = udp_listener.recv_from(&mut buf).await?;

            let mut res: Vec<u8> = Vec::with_capacity(n + 32);

            res.extend_from_slice(&[0, 0]);
            res.push(0);
            res.extend(SocksSocketAddr::from(peer_host).to_bytes());
            res.extend_from_slice(&buf[..n]);

            udp_listener.send_to(&res[..], source).await?;

            Ok(())
        };

        let res: crate::Result<_> = associate_inner().await;
        if let Err(Socks5Error::Socks5Error(err)) = res {
            self.reply(err, Default::default()).await?;
        }
        Ok(())
    }
}
