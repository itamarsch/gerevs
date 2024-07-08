use gerev::socks5_socket::{
    protocol::{addr::Addr, command::Command, reply::Reply},
    Sock5Socket,
};
use std::{
    error::Error,
    net::{SocketAddrV4, SocketAddrV6},
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (client, _addr) = server.accept().await?;
        println!("Connection");
        tokio::spawn(async move {
            let client = client;
            let mut client = Sock5Socket::new(client);
            let Ok((command, addr)) = client.socks_request().await else {
                return;
            };

            assert_eq!(command, Command::Connect);

            let result = match addr.addr {
                Addr::Ipv4(addrv4) => {
                    TcpStream::connect(SocketAddrV4::new(addrv4, addr.port)).await
                }
                Addr::Ipv6(addrv6) => {
                    TcpStream::connect(SocketAddrV6::new(addrv6, addr.port, 0, 0)).await
                }
                Addr::Domain(_) => todo!("Use dns-lookup"),
            };

            println!("Connected to server!");
            match result {
                Ok(mut server) => {
                    let result = client
                        .write_connect_reponse(Reply::Success, addr.clone())
                        .await;

                    if let Err(err) = result {
                        println!("Failed writing response: {:?}", err);
                        return;
                    }
                    if let Err(err) = tokio::io::copy_bidirectional(&mut client, &mut server).await
                    {
                        println!("Failed bidirectional: {:?}", err);
                    }
                    println!("Connectoin closed! hurray");
                }
                Err(_) => {
                    let _err = client
                        .write_connect_reponse(Reply::GeneralFailure, addr.clone())
                        .await;
                }
            };
        });
    }
}
