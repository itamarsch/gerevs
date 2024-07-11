use std::io;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

pub mod tunnel_bind;

pub trait Bind<C> {
    fn start_listening<T>(
        &mut self,
        server: &mut T,
        client: TcpStream,
        credentials: C,
    ) -> impl std::future::Future<Output = io::Result<()>> + Send
    where
        T: AsyncWrite + AsyncRead + Send + Unpin;
}
