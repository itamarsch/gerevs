use std::{future::Future, io};

use tokio::io::{AsyncRead, AsyncWrite};

pub use crate::protocol::AuthMethod;

mod no_auth_authenticator;
mod user_authenticator;

pub use no_auth_authenticator::*;
pub use user_authenticator::*;

pub trait Authenticator<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    type Credentials;

    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod;

    fn authenticate(
        &mut self,
        conn: &mut T,
        selected_method: AuthMethod,
    ) -> impl Future<Output = io::Result<Option<Self::Credentials>>> + Send;
}
