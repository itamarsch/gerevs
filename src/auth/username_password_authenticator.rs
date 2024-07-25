//! # User Authentication Module
//!
//! This module provides structures and traits for implementing username and password authentication
//! in the SOCKS5 protocol. It includes the `UsernamePasswordAuthenticator` struct for performing
//! the authentication and the `UserAuthenticator` trait for validating user credentials.
//!
//! ## Example
//!
//! ```rust
//! struct SimpleUserAuthenticator;
//!
//! impl UserAuthenticator for SimpleUserAuthenticator {
//!     type Credentials = ();
//!
//!     async fn authenticate_user(
//!         &mut self,
//!         user: User,
//!     ) -> io::Result<Option<Self::Credentials>> {
//!         if user.username == "admin" && user.password == "password" {
//!             Ok(Some(()))
//!         } else {
//!             Ok(None)
//!         }
//!     }
//! }
//!
//! let user_authenticator = SimpleUserAuthenticator;
//! let auth = UsernamePasswordAuthenticator::new(user_authenticator);
//! ```

use std::{
    future::Future,
    io::{self, ErrorKind},
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::debug;

use crate::protocol::AuthMethod;

use super::Authenticator;

const USER_PASSWORD_VERSION: u8 = 0x01;

/// Represents a user with a username and password.
#[derive(Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[repr(u8)]
enum AuthStatus {
    Success = 0x00,
    Failure = 0x01,
}

/// The `UserAuthenticator` trait defines the functionality for validating (authenticating) user credentials.
pub trait UserAuthenticator {
    type Credentials;

    /// Validates the provided user credentials and returns an optional `Credentials` value if
    /// the validation (authentication) is successful.
    ///
    /// - `user`: The user credentials to authenticate.
    /// - Returns a future that resolves to `io::Result<Option<Self::Credentials>>>`:
    ///   - `Ok(Some(credentials))`: Authentication was successful, and credentials are provided.
    ///   - `Ok(None)`: Authentication failed.
    ///   - `Err(error)`: An error occurred during the authentication process.
    fn authenticate_user(
        &mut self,
        user: User,
    ) -> impl Future<Output = io::Result<Option<Self::Credentials>>> + Send;
}

/// The `UsernamePasswordAuthenticator` struct handles the username and password authentication process
/// in the SOCKS5 protocol.
pub struct UsernamePasswordAuthenticator<U>
where
    U: UserAuthenticator,
{
    user_authenticator: U,
}

impl<T, U> Authenticator<T> for UsernamePasswordAuthenticator<U>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
    U: UserAuthenticator + Send + Sync,
    U::Credentials: Send,
{
    type Credentials = U::Credentials;

    /// Selects the `UsernamePassword` authentication method if it is present in the provided list
    /// of methods. If not, it selects `NoAcceptableMethods` to indicate that no suitable
    /// authentication method is available.
    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod {
        if methods.contains(&AuthMethod::UsernamePassword) {
            AuthMethod::UsernamePassword
        } else {
            AuthMethod::NoAcceptableMethods
        }
    }

    /// Performs the authentication process using the username and password method.
    async fn authenticate(
        &mut self,
        conn: &mut T,
        _: AuthMethod,
    ) -> io::Result<Option<Self::Credentials>> {
        let user = self.get_user(conn).await?;
        let credentials = self.user_authenticator.authenticate_user(user).await?;

        self.send_authentication_result(
            conn,
            if credentials.is_some() {
                AuthStatus::Success
            } else {
                AuthStatus::Failure
            },
        )
        .await?;

        Ok(credentials)
    }
}

impl<U> UsernamePasswordAuthenticator<U>
where
    U: UserAuthenticator,
{
    /// Creates a new `UsernamePasswordAuthenticator` with the provided user authenticator.
    ///
    /// - `user_authenticator`: The user authenticator that implements the `UserAuthenticator` trait.
    /// - Returns a new `UsernamePasswordAuthenticator`.
    pub fn new(user_authorizer: U) -> UsernamePasswordAuthenticator<U> {
        UsernamePasswordAuthenticator {
            user_authenticator: user_authorizer,
        }
    }

    async fn get_user<T>(&self, conn: &mut T) -> io::Result<User>
    where
        T: AsyncRead + Unpin,
    {
        let version = conn.read_u8().await?;
        if version != USER_PASSWORD_VERSION {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid UsernamePassword version",
            ));
        }

        let username_len = conn.read_u8().await?;
        if username_len < 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Username cannot be empty",
            ));
        }

        let mut buf = vec![0; username_len as usize];
        conn.read_exact(&mut buf).await?;
        let username = String::from_utf8(buf)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Username was invalid utf8"))?;

        let password_len = conn.read_u8().await?;
        if password_len < 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Password cannot be empty",
            ));
        }

        let mut buf = vec![0; password_len as usize];
        conn.read_exact(&mut buf).await?;
        let password = String::from_utf8(buf)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Password was invalid utf8"))?;

        debug!(
            "Received username: {:?}, and password: {:?}",
            username, password
        );
        let user = User { username, password };
        Ok(user)
    }

    async fn send_authentication_result<T>(
        &self,
        conn: &mut T,
        status: AuthStatus,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin,
    {
        conn.write_u8(USER_PASSWORD_VERSION).await?;
        conn.write_u8(status as u8).await?;
        Ok(())
    }
}
