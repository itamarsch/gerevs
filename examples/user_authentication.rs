use gerevs::{
    auth::username_password_authenticator::{UserAuthenticator, UsernamePasswordAuthenticator},
    method_handlers::{TunnelAssociate, TunnelBind, TunnelConnect},
    Socks5Socket,
};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, span, warn, Instrument, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let server = TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (client, _addr) = server.accept().await?;
        debug!("Received connection from: {:?}", _addr);
        let connection = span!(Level::INFO, "connection", %_addr);

        tokio::spawn(
            async move {
                let result = handle_connection(client).await;
                if let Err(err) = result {
                    warn!("Failed connection: {:?}", err);
                }
            }
            .instrument(connection),
        );
    }
}

async fn handle_connection(client: TcpStream) -> gerevs::Result<()> {
    let mut socks5_stream = Socks5Socket::new(
        client,
        UsernamePasswordAuthenticator::new(SimpleUserAuthenticator),
        TunnelConnect,
        TunnelBind,
        TunnelAssociate,
    );
    socks5_stream.run().await
}

struct SimpleUserAuthenticator;
impl UserAuthenticator for SimpleUserAuthenticator {
    type Credentials = ();

    async fn authenticate_user(
        &mut self,
        user: gerevs::auth::username_password_authenticator::User,
    ) -> std::io::Result<Option<Self::Credentials>> {
        if user.username == "itamar" && user.password == "password" {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}
