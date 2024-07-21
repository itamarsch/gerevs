use std::{fmt, io};

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

impl fmt::Display for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Reply::Success => "Success",
            Reply::GeneralFailure => "General Failure",
            Reply::ConnectionNotAllowedByRuleset => "Connection Not Allowed By Ruleset",
            Reply::NetworkUnreachable => "Network Unreachable",
            Reply::HostUnreachable => "Host Unreachable",
            Reply::ConnectionRefused => "Connection Refused",
            Reply::TTLExpired => "TTL Expired",
            Reply::CommandNotSupported => "Command Not Supported",
            Reply::AddressTypeNotSupported => "Address Type Not Supported",
        };
        write!(f, "{}", description)
    }
}

impl std::error::Error for Reply {}

impl Reply {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Reply::Success),
            0x01 => Some(Reply::GeneralFailure),
            0x02 => Some(Reply::ConnectionNotAllowedByRuleset),
            0x03 => Some(Reply::NetworkUnreachable),
            0x04 => Some(Reply::HostUnreachable),
            0x05 => Some(Reply::ConnectionRefused),
            0x06 => Some(Reply::TTLExpired),
            0x07 => Some(Reply::CommandNotSupported),
            0x08 => Some(Reply::AddressTypeNotSupported),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl Reply {
    pub fn from_io_result<T>(value: &io::Result<T>) -> Self {
        match value {
            Ok(_) => Reply::Success,
            Err(err) => err.kind().into(),
        }
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
