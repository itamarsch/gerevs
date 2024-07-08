use gerev::{
    addr::{Addr, AddressType, SocksSocketAddr},
    command::Command,
    methods::AuthMethod,
    reply::Reply,
};
use std::{
    error::Error,
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const VERSION: u8 = 0x05;
const RESERVED: u8 = 0x00;

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

async fn socks_request(client: &mut TcpStream) -> io::Result<(Command, SocksSocketAddr)> {
    let methods = parse_methods(client).await?;

    if methods.contains(&AuthMethod::NoAuthRequired) {
        write_auth_method(client, AuthMethod::NoAuthRequired).await?;
    }

    let (command, addr_type) = parse_request(client).await?;
    let addr = parse_addr(client, addr_type).await?;
    println!(
        "Methods {:?}, Command: {:?}, AddrType: {:?}, Addr: {:?}",
        methods, command, addr_type, addr
    );

    Ok((command, addr))
}

async fn write_connect_reponse<T>(
    stream: &mut T,
    reply: Reply,
    bnd_address: SocksSocketAddr,
) -> io::Result<()>
where
    T: AsyncWrite + Unpin,
{
    stream.write_u8(VERSION).await?;

    stream.write_u8(reply.to_u8()).await?;

    stream.write_u8(RESERVED).await?;

    stream.write_all(&bnd_address.to_bytes()).await?;

    Ok(())
}

async fn write_auth_method<T>(stream: &mut T, auth_method: AuthMethod) -> io::Result<()>
where
    T: AsyncWrite + Unpin,
{
    stream.write_u8(VERSION).await?;
    stream.write_u8(auth_method.to_u8()).await?;
    Ok(())
}

async fn parse_methods<T>(stream: &mut T) -> io::Result<Vec<AuthMethod>>
where
    T: AsyncRead + Unpin,
{
    let mut header: [u8; 2] = [0; 2];
    stream.read_exact(&mut header).await?;

    assert_eq!(header[0], VERSION);

    let mut methods = vec![0; header[1] as usize];

    stream.read_exact(&mut methods).await?;
    let methods = methods
        .into_iter()
        .map(AuthMethod::from_u8)
        .collect::<Vec<_>>();

    Ok(methods)
}

async fn parse_request<T>(stream: &mut T) -> io::Result<(Command, AddressType)>
where
    T: AsyncRead + Unpin,
{
    let mut request: [u8; 4] = [0; 4];
    stream.read_exact(&mut request).await?;
    assert_eq!(request[0], VERSION);
    assert_eq!(request[2], RESERVED);
    let command = Command::from_u8(request[1]);
    let addr_type = AddressType::from_u8(request[3]);
    Ok((command, addr_type))
}

async fn parse_addr<T>(stream: &mut T, address: AddressType) -> io::Result<SocksSocketAddr>
where
    T: AsyncRead + Unpin,
{
    let addr = match address {
        AddressType::Ipv4 => {
            let mut addr = [0; 4];
            stream.read_exact(&mut addr).await?;
            Addr::Ipv4(Ipv4Addr::from(addr))
        }
        AddressType::DomainName => {
            let len = stream.read_u8().await?;
            let mut domain = vec![0; len as usize];
            stream.read_exact(&mut domain[..]).await?;
            let domain = String::from_utf8(domain).map_err(|_| io::ErrorKind::InvalidData)?;
            Addr::Domain(domain)
        }
        AddressType::Ipv6 => {
            let mut addr = [0; 16];
            stream.read_exact(&mut addr).await?;
            Addr::Ipv6(Ipv6Addr::from(addr))
        }
    };
    let port = stream.read_u16().await?;

    Ok(SocksSocketAddr { port, addr })
}
