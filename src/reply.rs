#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Reply {
    Success = 0x00,
    GeneralFailure = 0x01,
    ConnectionNotAllowedByRuleset = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TTLExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
}

impl Reply {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => Reply::Success,
            0x01 => Reply::GeneralFailure,
            0x02 => Reply::ConnectionNotAllowedByRuleset,
            0x03 => Reply::NetworkUnreachable,
            0x04 => Reply::HostUnreachable,
            0x05 => Reply::ConnectionRefused,
            0x06 => Reply::TTLExpired,
            0x07 => Reply::CommandNotSupported,
            0x08 => Reply::AddressTypeNotSupported,
            _ => panic!("Invalid value for Reply"),
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
