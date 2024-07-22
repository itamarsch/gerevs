mod addr;
mod command;
mod methods;
mod reply;

pub use addr::Addr;
pub use addr::AddressType;
pub use addr::SocksSocketAddr;
pub use command::Command;
pub use methods::AuthMethod;
pub use reply::Reply;

pub const VERSION: u8 = 0x05;
pub const RESERVED: u8 = 0x00;
pub const RESERVED_16: u16 = 0x00;
