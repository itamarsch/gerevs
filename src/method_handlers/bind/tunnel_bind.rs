use super::Bind;

pub struct TunnelBind;

impl Bind<()> for TunnelBind {
    async fn start_listening<T>(
        &mut self,
        server: &mut T,
        mut client: tokio::net::TcpStream,
        _: (),
    ) -> std::io::Result<()>
    where
        T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin,
    {
        tokio::io::copy_bidirectional(server, &mut client)
            .await
            .map(|_| ())
    }
}
