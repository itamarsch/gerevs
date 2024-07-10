use std::io;

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

impl From<io::ErrorKind> for Reply {
    fn from(error_kind: io::ErrorKind) -> Self {
        match error_kind {
            // Stable variants
            io::ErrorKind::NotFound => Reply::HostUnreachable, // Assuming HostUnreachable for file not found
            io::ErrorKind::PermissionDenied => Reply::ConnectionNotAllowedByRuleset,
            io::ErrorKind::ConnectionRefused => Reply::ConnectionRefused,
            io::ErrorKind::ConnectionReset => Reply::TTLExpired, // Assuming TTLExpired for connection reset
            io::ErrorKind::ConnectionAborted => Reply::TTLExpired, // Assuming TTLExpired for connection aborted
            io::ErrorKind::NotConnected => Reply::NetworkUnreachable, // Assuming NetworkUnreachable for not connected
            io::ErrorKind::AddrInUse => Reply::HostUnreachable, // Assuming HostUnreachable for address in use
            io::ErrorKind::AddrNotAvailable => Reply::AddressTypeNotSupported,
            io::ErrorKind::BrokenPipe => Reply::TTLExpired, // Assuming TTLExpired for broken pipe
            io::ErrorKind::AlreadyExists => Reply::HostUnreachable, // Assuming HostUnreachable for already exists
            io::ErrorKind::WouldBlock => Reply::HostUnreachable, // Assuming HostUnreachable for would block
            io::ErrorKind::InvalidInput => Reply::HostUnreachable, // Assuming HostUnreachable for invalid input
            io::ErrorKind::InvalidData => Reply::HostUnreachable, // Assuming HostUnreachable for invalid data
            io::ErrorKind::TimedOut => Reply::TTLExpired,
            io::ErrorKind::WriteZero => Reply::HostUnreachable, // Assuming HostUnreachable for write zero
            io::ErrorKind::Interrupted => Reply::HostUnreachable, // Assuming HostUnreachable for interrupted
            io::ErrorKind::Unsupported => Reply::CommandNotSupported,
            io::ErrorKind::UnexpectedEof => Reply::HostUnreachable, // Assuming HostUnreachable for unexpected EOF
            _ => Reply::GeneralFailure,
        }
    }
}

fn main() {
    let error_kind = io::ErrorKind::ConnectionRefused;
    let reply: Reply = error_kind.into();
    println!("Reply code: {:?}", reply as u8);
}
