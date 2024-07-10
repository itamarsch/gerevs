use gerev::{
    auth::no_auth_authenticator::NoAuthAuthenticator,
    method_handlers::TunnelConnect,
    protocol::{Addr, Reply},
    socks5_socket::Sock5Socket,
};
use std::{
    error::Error,
    io,
    net::{SocketAddrV4, SocketAddrV6},
};
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
    let mut sock5_stream = Sock5Socket::new(client, NoAuthAuthenticator, TunnelConnect);
    let (command, addr, credentials) = sock5_stream.socks_request().await?;
    println!("Connection, addr: {:?}", addr);
    match command {
        gerev::protocol::Command::Connect => sock5_stream.connect(addr, credentials).await?,
        gerev::protocol::Command::Bind => {
            let res = match addr.addr {
                Addr::Ipv4(addrv4) => TcpListener::bind(SocketAddrV4::new(addrv4, addr.port)).await,
                Addr::Ipv6(addrv6) => {
                    TcpListener::bind(SocketAddrV6::new(addrv6, addr.port, 0, 0)).await
                }
                Addr::Domain(ref domain) => {
                    let domain = format!("{}:{}", domain, addr.port);
                    TcpListener::bind(domain).await
                }
            };

            let server = match res {
                Ok(server) => server,
                Err(err) => {
                    sock5_stream
                        .write_connect_reponse(err.kind().into(), addr)
                        .await?;
                    return Err(err);
                }
            };

            let localaddr = server.local_addr()?;
            println!("Local addr: {:?}", localaddr);
            sock5_stream
                .write_connect_reponse(Reply::Success, localaddr.into())
                .await?;

            let res = server.accept().await;
            let (mut client, client_addr) = match res {
                Ok(client) => client,
                Err(err) => {
                    sock5_stream
                        .write_connect_reponse(err.kind().into(), addr)
                        .await?;

                    return Err(err);
                }
            };

            sock5_stream
                .write_connect_reponse(Reply::Success, client_addr.into())
                .await?;
            tokio::io::copy_bidirectional(&mut client, &mut sock5_stream).await?;
        }
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
