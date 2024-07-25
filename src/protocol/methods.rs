const NO_AUTH_REQUIRED: u8 = 0x00;
const GSSAPI: u8 = 0x01;
const USERNAME_PASSWORD: u8 = 0x02;

const IANA_ASSIGNED_LOWER: u8 = 0x03;
const IANA_ASSIGNED_UPPER: u8 = 0x7F;

const PRIVATE_METHOD_LOWER: u8 = 0x80;
const PRIVATE_METHOD_UPPER: u8 = 0xFE;

const NO_ACCEPTABLE_METHODS: u8 = 0xFF;

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AuthMethod {
    NoAuthRequired,
    Gssapi,
    UsernamePassword,
    IanaAssigned(u8),
    PrivateMethods(u8),
    NoAcceptableMethods,
}

impl AuthMethod {
    pub(crate) fn from_u8(value: u8) -> Self {
        match value {
            NO_AUTH_REQUIRED => AuthMethod::NoAuthRequired,
            GSSAPI => AuthMethod::Gssapi,
            USERNAME_PASSWORD => AuthMethod::UsernamePassword,
            (IANA_ASSIGNED_LOWER..=IANA_ASSIGNED_UPPER) => AuthMethod::IanaAssigned(value),
            PRIVATE_METHOD_LOWER..=PRIVATE_METHOD_UPPER => AuthMethod::PrivateMethods(value),
            _ => unreachable!("u8 range handled fully"),
        }
    }
    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            AuthMethod::NoAuthRequired => NO_AUTH_REQUIRED,
            AuthMethod::Gssapi => GSSAPI,
            AuthMethod::UsernamePassword => USERNAME_PASSWORD,
            AuthMethod::IanaAssigned(value) => value,
            AuthMethod::PrivateMethods(value) => value,
            AuthMethod::NoAcceptableMethods => NO_ACCEPTABLE_METHODS,
        }
    }
}
