use std::{future::Future, io};

use tokio::io::{AsyncRead, AsyncWrite};

pub use crate::protocol::AuthMethod;

mod no_auth_authenticator;
pub mod user_authenticator;

pub use no_auth_authenticator::NoAuthAuthenticator;

/// # Authenticator Trait
///
/// The `Authenticator` trait defines the necessary functionality for handling authentication
/// in the SOCKS5 protocol. Implementations of this trait specify how to select an authentication
/// method and perform the actual authentication process.
///
/// ## Type Parameters
///
/// - `T`: The type of connection, which must implement `AsyncRead`, `AsyncWrite`, `Unpin`, and `Send`.
pub trait Authenticator<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    /// The type of credentials produced by the authentication process, if any.
    type Credentials;

    /// This method is responsible for selecting an authentication method from a list of supported methods
    /// provided by the client. It returns the selected `AuthMethod`.
    ///
    /// - `methods`: A slice of `AuthMethod` indicating the methods supported by the client.
    /// - Returns the selected `AuthMethod`.
    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod;

    /// This method performs the authentication process over the provided connection using the selected
    /// authentication method. It returns a future that resolves to an `io::Result` containing an optional
    /// `Credentials` value.
    ///
    /// - `conn`: A mutable reference to the connection `T`.
    /// - `selected_method`: The `AuthMethod` that was selected during the method selection phase.
    /// - Returns a future that resolves to `io::Result<Option<Self::Credentials>>>`:
    ///   - `Ok(Some(credentials))`: Authentication was successful, and credentials are provided.
    ///   - `Ok(None)`: Authentication failed.
    ///   - `Err(error)`: An error occurred during the authentication sub-negotiation process.
    fn authenticate(
        &mut self,
        conn: &mut T,
        selected_method: AuthMethod,
    ) -> impl Future<Output = io::Result<Option<Self::Credentials>>> + Send;
}
