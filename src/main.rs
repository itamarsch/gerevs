use gerev::{
    addr::Addr,
    protocol::{socks_request, write_connect_reponse},
    reply::Reply,
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
            let mut client = client;
            let Ok((command, addr)) = socks_request(&mut client).await else {
                return;
            };

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
                    let result =
                        write_connect_reponse(&mut client, Reply::Success, addr.clone()).await;
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
                    let _err =
                        write_connect_reponse(&mut client, Reply::GeneralFailure, addr.clone())
                            .await;
                }
            };
        });
    }
}
