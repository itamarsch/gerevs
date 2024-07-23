use gerevs::{
    auth::no_auth_authenticator::NoAuthAuthenticator,
    method_handlers::{associate_denier::AssociateDenier, BindDenier, TunnelConnect},
    socks5_socket::Sock5Socket,
};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (client, _addr) = server.accept().await?;
        tokio::spawn(async move {
            let result = handle_connection(client).await;
            if let Err(err) = result {
                eprintln!("Failed connection: {:?}", err);
            } else {
                println!("Connectoin closed! hurray");
            }
        });
    }
}

async fn handle_connection(client: TcpStream) -> gerevs::Result<()> {
    let mut sock5_stream = Sock5Socket::new(
        client,
        NoAuthAuthenticator,
        TunnelConnect,
        BindDenier,
        AssociateDenier,
    );
    sock5_stream.run().await
}
