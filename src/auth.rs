use std::{future::Future, io};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::AuthMethod;

pub mod no_auth_authenticator;
pub mod simple_user_authenticator;
pub mod user_authenticator;

pub trait Authenticator<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    type Credentials;
    /// Selects which method to use
    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod;

    /// Authenticator Should:
    /// Uses `conn` for comunication,
    /// perform all the communication for authentication
    /// return Some(credentials) on success
    /// return None on auth failure
    fn authenticate(
        &mut self,
        conn: &mut T,
    ) -> impl Future<Output = io::Result<Option<Self::Credentials>>> + Send;
}
