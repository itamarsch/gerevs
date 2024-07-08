use std::future::Future;

use tokio::io::{AsyncRead, AsyncWrite};

use super::protocol::methods::AuthMethod;

pub mod no_auth_authenticator;

pub trait Authenticator<T, R>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    /// Selects which method to use
    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod;

    /// Authenticator Should:
    /// Uses `conn` for comunication,
    /// perform all the communication for authentication
    /// return Some(credentials) on success
    /// return None on auth failure
    fn authenticate(&mut self, conn: &mut T) -> impl Future<Output = Option<R>> + Send;
}
