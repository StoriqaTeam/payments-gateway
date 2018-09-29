use client::ErrorKind as ClientErrorKind;
use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "controller error - unauthorized")]
    Unauthorized,
    #[fail(display = "controller error - bad request")]
    BadRequest,
    #[fail(display = "controller error - internal error")]
    Internal,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "controller source - error inside of Hyper library")]
    Hyper,
    #[fail(display = "controller source - error parsing config data")]
    Config,
    #[fail(display = "controller source - error fetching data using Storiqa client")]
    StoriqaClient,
    #[fail(display = "controller source - error converting json data from request")]
    RequestJson,
    #[fail(display = "controller source - error parsing bytes into utf8 from request")]
    RequestUTF8,
    #[fail(display = "controller source - error converting json data from request")]
    ResponseJson,
    #[fail(display = "controller source - error in jwt authentication")]
    JwtAuth,
    #[fail(display = "controller source - no source")]
    NoSource,
}

#[allow(dead_code)]
impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

impl From<ClientErrorKind> for ErrorKind {
    fn from(err: ClientErrorKind) -> Self {
        match err {
            ClientErrorKind::Internal => ErrorKind::Internal,
            ClientErrorKind::Unauthorized => ErrorKind::Unauthorized,
            ClientErrorKind::MalformedInput => ErrorKind::BadRequest,
        }
    }
}
