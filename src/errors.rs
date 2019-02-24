use std::io::Error as IoError;
use std::time::SystemTimeError;

use biscuit::errors::Error as BiscuitError;
use pem::errors::Error as PemError;
use reqwest::Error as ReqwestError;
use ring::error::Unspecified;
use serde_json::error::Error as SerdeJsonError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(IoError),
    Token(BiscuitError),
    Time(SystemTimeError),
    Pem(PemError),
    Security,
    SerdeJson(SerdeJsonError),
    Service(u16),
    Http(ReqwestError),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

impl From<BiscuitError> for Error {
    fn from(error: BiscuitError) -> Self {
        From::from(ErrorKind::Token(error))
    }
}

impl From<SystemTimeError> for Error {
    fn from(error: SystemTimeError) -> Self {
        From::from(ErrorKind::Time(error))
    }
}

impl From<PemError> for Error {
    fn from(error: PemError) -> Self {
        From::from(ErrorKind::Pem(error))
    }
}

impl From<Unspecified> for Error {
    fn from(_: Unspecified) -> Self {
        From::from(ErrorKind::Security)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(error: SerdeJsonError) -> Self {
        From::from(ErrorKind::SerdeJson(error))
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        From::from(ErrorKind::Io(error))
    }
}

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Self {
        From::from(ErrorKind::Http(error))
    }
}
