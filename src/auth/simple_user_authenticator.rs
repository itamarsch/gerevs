use std::io;

use super::user_authenticator::{User, UserAuthenticator, UserValidator};

pub struct SingleUserValidator {
    valid_username: String,
    valid_password: String,
}

impl UserValidator<()> for SingleUserValidator {
    async fn validate_user(&mut self, user: User) -> io::Result<Option<()>> {
        if user.username == self.valid_username && user.password == self.valid_password {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

pub fn simple_user_authenticator() -> UserAuthenticator<(), SingleUserValidator> {
    UserAuthenticator::new(SingleUserValidator {
        valid_username: "itamar".into(),
        valid_password: "schwartz".into(),
    })
}
