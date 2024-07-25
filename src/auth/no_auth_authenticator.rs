use std::io;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::AuthMethod;

use super::Authenticator;

/// The `NoAuthAuthenticator` struct is an implementation of the `Authenticator` trait that handles
/// the "no authentication" method in the SOCKS5 protocol. It requires no credentials from the client
/// and performs no sub-negotiation.
pub struct NoAuthAuthenticator;

impl<T> Authenticator<T> for NoAuthAuthenticator
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    /// The type of credentials produced by the authentication process, which is `()` in this case.
    type Credentials = ();

    /// This method performs the "no authentication" process, which is essentially a no-op in this case.
    /// It immediately returns `Ok(Some(()))` to indicate successful authentication with no credentials.
    async fn authenticate(&mut self, _: &mut T, _: AuthMethod) -> io::Result<Option<()>> {
        Ok(Some(()))
    }

    /// This method selects the `NoAuthRequired` method if it is present in the provided list of methods.
    /// If not, it selects `NoAcceptableMethods` to indicate that no suitable authentication method is available.
    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod {
        if methods.contains(&AuthMethod::NoAuthRequired) {
            AuthMethod::NoAuthRequired
        } else {
            AuthMethod::NoAcceptableMethods
        }
    }
}
