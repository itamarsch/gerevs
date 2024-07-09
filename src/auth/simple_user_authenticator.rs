use std::{future::Future, pin::Pin};

use super::user_authemticator::{User, UserAuthenticator};

fn validate_user(user: User) -> Pin<Box<dyn Future<Output = Option<()>> + Send>> {
    Box::pin(async move {
        if user.username == "admin1" && user.password == "hi" {
            Some(())
        } else {
            None
        }
    })
}

pub fn simple_user_authenticator(
) -> UserAuthenticator<(), impl FnMut(User) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>> {
    UserAuthenticator::new(validate_user)
}
