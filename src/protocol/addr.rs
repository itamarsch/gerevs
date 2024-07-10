use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

#[derive(Debug, Clone, Copy)]
pub enum AddressType {
    Ipv4 = 0x01,
    DomainName = 0x03,
    Ipv6 = 0x04,
}

impl AddressType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => AddressType::Ipv4,
            0x03 => AddressType::DomainName,
            0x04 => AddressType::Ipv6,
            _ => panic!("Invalid value for AddressType"),
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
impl SocksSocketAddr {
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
