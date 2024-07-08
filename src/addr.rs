use std::net::{Ipv4Addr, Ipv6Addr};

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
#[derive(Debug)]
pub struct SocksSocketAddr {
    pub port: u16,
    pub addr: Addr,
}

#[derive(Debug)]
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
