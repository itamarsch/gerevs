use std::{
    future::Future,
    io::{self, ErrorKind},
    marker::PhantomData,
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::trace;

use crate::protocol::AuthMethod;

use super::Authenticator;

const USER_PASSWORD_VERSION: u8 = 0x01;

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

pub trait UserValidator<R> {
    fn validate_user(&mut self, user: User) -> impl Future<Output = io::Result<Option<R>>> + Send;
}

pub struct UserAuthenticator<R, U>
where
    U: UserValidator<R>,
{
    phantom_data_credentials: PhantomData<R>,
    user_validator: U,
}

impl<T, R, U> Authenticator<T> for UserAuthenticator<R, U>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
    R: Send + Sync,
    U: UserValidator<R> + Send + Sync,
{
    type Credentials = R;

    async fn authenticate(&mut self, conn: &mut T) -> io::Result<Option<R>> {
        let user = self.get_user(conn).await?;

        let credentials = self.user_validator.validate_user(user).await?;

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

    fn select_method(&self, methods: &[AuthMethod]) -> AuthMethod {
        if methods.contains(&AuthMethod::UsernamePassword) {
            AuthMethod::UsernamePassword
        } else {
            AuthMethod::NoAcceptableMethods
        }
    }
}

impl<R, U> UserAuthenticator<R, U>
where
    U: UserValidator<R>,
{
    pub fn new(user_validator: U) -> UserAuthenticator<R, U> {
        UserAuthenticator {
            phantom_data_credentials: PhantomData,
            user_validator,
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

        trace!(
            "Received username: {:?}, and password: {:?}",
            username,
            password
        );
        let user: User = User { username, password };
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
