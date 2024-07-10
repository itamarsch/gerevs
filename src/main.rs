use gerev::{
    auth::{
        no_auth_authenticator::NoAuthAuthenticator,
        simple_user_authenticator::simple_user_authenticator,
    },
    protocol::{Addr, Command, Reply},
    socks5_socket::Sock5Socket,
};
use std::{
    error::Error,
    io,
    net::{SocketAddrV4, SocketAddrV6, ToSocketAddrs},
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (client, _addr) = server.accept().await?;
        println!("Connection");
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
    let mut client = Sock5Socket::new(client, NoAuthAuthenticator);
    let (command, addr, ()) = client.socks_request().await?;
    assert_eq!(command, Command::Connect);
    println!("Command: {:?}, Addr: {:?}", command, addr);

    let result = match addr.addr {
        Addr::Ipv4(addrv4) => TcpStream::connect(SocketAddrV4::new(addrv4, addr.port)).await,
        Addr::Ipv6(addrv6) => TcpStream::connect(SocketAddrV6::new(addrv6, addr.port, 0, 0)).await,
        Addr::Domain(ref domain) => {
            let domain = format!("{}:{}", domain, addr.port);
            TcpStream::connect(domain).await
        }
    };

    match result {
        Ok(mut server) => {
            println!("Connected to server!");
            client
                .write_connect_reponse(Reply::Success, addr.clone())
                .await?;

            tokio::io::copy_bidirectional(&mut client, &mut server).await?;
        }
        Err(err) => {
            client
                .write_connect_reponse(err.kind().into(), addr.clone())
                .await?;
        }
    };
    Ok(())
}
