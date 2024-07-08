use gerev::{
    addr::{Addr, AddressType, SocksSocketAddr},
    command::Command,
    methods::AuthMethod,
};
use std::{
    error::Error,
    net::{Ipv4Addr, Ipv6Addr},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

const VERSION: u8 = 0x05;
const RESERVED: u8 = 0x00;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (mut client, _addr) = server.accept().await?;
        println!("Connection");

        let methods = parse_methods(&mut client).await?;

        if methods.contains(&AuthMethod::NoAuthRequired) {
            client.write_u8(VERSION).await?;
            client.write_u8(AuthMethod::NoAuthRequired.to_u8()).await?;
        }

        let (command, addr_type) = parse_request(&mut client).await?;
        let addr = parse_addr(&mut client, addr_type).await?;
        println!(
            "Methods {:?}, Command: {:?}, AddrType: {:?}, Addr: {:?}",
            methods, command, addr_type, addr
        );
    }
}

async fn parse_methods<T>(stream: &mut T) -> Result<Vec<AuthMethod>, Box<dyn Error>>
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
async fn parse_request<T>(stream: &mut T) -> Result<(Command, AddressType), Box<dyn Error>>
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

async fn parse_addr<T>(
    stream: &mut T,
    address: AddressType,
) -> Result<SocksSocketAddr, Box<dyn Error>>
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
            let domain = String::from_utf8(domain)?;
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
