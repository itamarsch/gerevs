use std::{future::Future, pin::Pin};

use super::user_authemticator::{User, UserAuthenticator};

fn validate_user(user: User) -> Pin<Box<dyn Future<Output = Option<()>> + Send>> {
    Box::pin(async move {
        println!("User: {:?}", user);
        None
    })
}

pub fn simple_user_authenticator(
) -> UserAuthenticator<(), impl FnMut(User) -> Pin<Box<dyn Future<Output = Option<()>> + Send>>> {
    UserAuthenticator::new(validate_user)
}
