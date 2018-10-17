use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use validator::ValidationErrors;

use client::storiqa::ErrorKind as StoriqaClientErrorKind;
use client::transactions::ErrorKind as TransactionsClientErrorKind;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "service error - unauthorized")]
    Unauthorized,
    #[fail(display = "service error - malformed input")]
    MalformedInput,
    #[fail(display = "service error - invalid input, errors: {}", _0)]
    InvalidInput(ValidationErrors),
    #[fail(display = "service error - internal error")]
    Internal,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "service error context - internal error")]
    Internal,
    #[fail(display = "jwt auth error - error inside json web token crate")]
    JsonWebToken,
}

#[allow(dead_code)]
impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
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

impl From<StoriqaClientErrorKind> for ErrorKind {
    fn from(err: StoriqaClientErrorKind) -> Self {
        match err {
            StoriqaClientErrorKind::Internal => ErrorKind::Internal,
            StoriqaClientErrorKind::Unauthorized => ErrorKind::Unauthorized,
            StoriqaClientErrorKind::MalformedInput => ErrorKind::MalformedInput,
        }
    }
}

impl From<TransactionsClientErrorKind> for ErrorKind {
    fn from(err: TransactionsClientErrorKind) -> Self {
        match err {
            TransactionsClientErrorKind::Internal => ErrorKind::Internal,
            TransactionsClientErrorKind::Unauthorized => ErrorKind::Unauthorized,
            TransactionsClientErrorKind::MalformedInput => ErrorKind::MalformedInput,
        }
    }
}
