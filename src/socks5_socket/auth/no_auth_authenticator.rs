use tokio::net::TcpStream;

use crate::socks5_socket::protocol::methods::AuthMethod;

use super::Authenticator;

pub struct NoAuthAuthenticator;

impl Authenticator<TcpStream, ()> for NoAuthAuthenticator {
    async fn authenticate(&mut self, _: &mut TcpStream) -> Option<()> {
        Some(())
    }

    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod {
        if methods.contains(&AuthMethod::NoAuthRequired) {
            AuthMethod::NoAuthRequired
        } else {
            AuthMethod::NoAcceptableMethods
        }
    }
}
