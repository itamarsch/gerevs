use gerev::{
    auth::no_auth_authenticator::NoAuthAuthenticator,
    method_handlers::{TunnelBind, TunnelConnect},
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

async fn handle_connection(client: TcpStream) -> gerev::Result<()> {
    let mut sock5_stream = Sock5Socket::new(client, NoAuthAuthenticator, TunnelConnect, TunnelBind);
    let (command, addr, credentials) = sock5_stream.socks_request().await?;
    println!("Connection, addr: {:?}", addr);
    match command {
        gerev::protocol::Command::Connect => sock5_stream.connect(addr, credentials).await?,
        gerev::protocol::Command::Bind => sock5_stream.bind(addr, credentials).await?,
        gerev::protocol::Command::UdpAssociate => todo!(),
    }

    Ok(())
}
