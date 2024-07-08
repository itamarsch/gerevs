#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

impl Command {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => Command::Connect,
            0x02 => Command::Bind,
            0x03 => Command::UdpAssociate,
            _ => panic!("Invalid value for Command"),
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
