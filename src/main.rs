use gerev::{
    auth::no_auth_authenticator::NoAuthAuthenticator, method_handlers::TunnelConnect,
    socks5_socket::Sock5Socket,
};
use std::{error::Error, io};
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

async fn handle_connection(client: TcpStream) -> io::Result<()> {
    let mut client = Sock5Socket::new(client, NoAuthAuthenticator, TunnelConnect);
    let (command, addr, credentials) = client.socks_request().await?;
    println!("Connection, addr: {:?}", addr);
    match command {
        gerev::protocol::Command::Connect => client.connect(addr, credentials).await?,
        gerev::protocol::Command::Bind => todo!(),
        gerev::protocol::Command::UdpAssociate => todo!(),
    }

    // match result {
    //     Ok(mut server) => {
    //         println!("Connected to server!");
    //         client
    //             .write_connect_reponse(Reply::Success, addr.clone())
    //             .await?;
    //     }
    //     Err(err) => {
    //         client
    //             .write_connect_reponse(err.kind().into(), addr.clone())
    //             .await?;
    //     }
    // };

    Ok(())
}
