use std::io;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::AuthMethod;

use super::{user_authemticator::UserAuthenticator, Authenticator};

pub struct NoAuthAuthenticator;

impl<T> Authenticator<T, ()> for NoAuthAuthenticator
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn authenticate(&mut self, _: &mut T) -> io::Result<Option<()>> {
        Ok(Some(()))
    }

    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod {
        if methods.contains(&AuthMethod::NoAuthRequired) {
            AuthMethod::NoAuthRequired
        } else {
            AuthMethod::NoAcceptableMethods
        }
    }
}
