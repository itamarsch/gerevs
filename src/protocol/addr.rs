use std::{
    io::{self, ErrorKind},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
    ops::Deref,
};

use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Clone, Copy)]
pub enum AddressType {
    Ipv4 = 0x01,
    DomainName = 0x03,
    Ipv6 = 0x04,
}

impl AddressType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(AddressType::Ipv4),
            0x03 => Some(AddressType::DomainName),
            0x04 => Some(AddressType::Ipv6),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
#[derive(Debug, Clone)]
pub struct SocksSocketAddr {
    pub port: u16,
    pub addr: Addr,
}
impl Default for SocksSocketAddr {
    fn default() -> Self {
        Self {
            port: 0,
            addr: Addr::Ipv4(Ipv4Addr::from([0; 4])),
        }
    }
}

impl SocksSocketAddr {
    pub async fn to_socket_addr(&self) -> io::Result<impl Deref<Target = [SocketAddr]>> {
        match self.addr {
            Addr::Ipv4(addrv4) => Ok(vec![SocketAddrV4::new(addrv4, self.port).into()]),
            Addr::Ipv6(addrv6) => Ok(vec![SocketAddrV6::new(addrv6, self.port, 0, 0).into()]),
            Addr::Domain(ref domain) => {
                let domain = format!("{}:{}", domain, self.port);

                Ok(
                    tokio::task::spawn_blocking(move || domain.to_socket_addrs())
                        .await
                        .expect("Task isn't aborted")?
                        .collect(),
                )
            }
        }
    }
    pub async fn read<T>(stream: &mut T) -> io::Result<Self>
    where
        T: AsyncRead + Unpin,
    {
        let address_type = stream.read_u8().await?;
        let Some(address_type) = AddressType::from_u8(address_type) else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid address type",
            ));
        };

        let addr = match address_type {
            AddressType::Ipv4 => {
                let mut addr = [0; 4];
                stream.read_exact(&mut addr).await?;
                Addr::Ipv4(Ipv4Addr::from(addr))
            }
            AddressType::DomainName => {
                let len = stream.read_u8().await?;
                let mut domain = vec![0; len as usize];
                stream.read_exact(&mut domain[..]).await?;
                let domain = String::from_utf8(domain).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Domain name was invalid utf8")
                })?;
                Addr::Domain(domain)
            }
            AddressType::Ipv6 => {
                let mut addr = [0; 16];
                stream.read_exact(&mut addr).await?;
                Addr::Ipv6(Ipv6Addr::from(addr))
            }
        };
        let port = stream.read_u16().await?;
        Ok(Self { port, addr })
    }
    /// Turns `Self` into: AddrType+ADDR+PORT
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(18);

        bytes.push(self.addr.addr_type().to_u8());

        match &self.addr {
            Addr::Ipv4(addr) => bytes.extend_from_slice(&addr.octets()[..]),
            Addr::Ipv6(addr) => bytes.extend_from_slice(&addr.octets()[..]),
            Addr::Domain(domain) => {
                assert!(domain.len() < 256);
                bytes.push(domain.len() as u8);
                bytes.extend_from_slice(domain.as_bytes())
            }
        }
        bytes.extend_from_slice(&self.port.to_be_bytes());

        bytes
    }
}

impl From<SocketAddr> for SocksSocketAddr {
    fn from(value: SocketAddr) -> Self {
        match value {
            SocketAddr::V4(ipv4) => SocksSocketAddr {
                port: ipv4.port(),
                addr: Addr::Ipv4(*ipv4.ip()),
            },
            SocketAddr::V6(ipv6) => SocksSocketAddr {
                port: ipv6.port(),
                addr: Addr::Ipv6(*ipv6.ip()),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Addr {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
    Domain(String),
}
impl Addr {
    pub fn addr_type(&self) -> AddressType {
        match self {
            Addr::Ipv4(_) => AddressType::Ipv4,
            Addr::Ipv6(_) => AddressType::Ipv6,
            Addr::Domain(_) => AddressType::DomainName,
        }
    }
}
