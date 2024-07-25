use std::io;

use super::Bind;
/// The `TunnelBind` struct is an implementation of the `Bind` trait that handles TCP BIND requests
/// by establishing a TCP connection and relaying data between the client and the target server.
///
/// This is a simple and basic implementation that binds to a local address and directly relays
/// TCP packets between the client and the target server without any additional processing or filtering.
///
/// This struct can be used in scenarios where basic TCP traffic needs to be tunneled through
/// a SOCKS5 proxy server without any special handling or configuration.
pub struct TunnelBind;

impl Bind<()> for TunnelBind {
    async fn start_listening<T>(
        &mut self,
        server: &mut T,
        mut client: tokio::net::TcpStream,
        _: (),
    ) -> crate::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        let res = tokio::io::copy_bidirectional(server, &mut client)
            .await
            .map(|_| ());

        if let Err(err) = &res {
            if matches!(err.kind(), io::ErrorKind::NotConnected) {
                return Ok(());
            }
        }

        res?;
        Ok(())
    }
}
