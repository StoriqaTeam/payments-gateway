use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};

use services::ErrorKind as ServiceErrorKind;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "controller error - unauthorized")]
    Unauthorized,
    #[fail(display = "controller error - bad request")]
    BadRequest,
    #[fail(display = "controller error - unprocessable entity")]
    UnprocessableEntity(String),
    #[fail(display = "controller error - internal error")]
    Internal,
    #[fail(display = "controller error - not found")]
    NotFound,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "controller source - error inside of Hyper library")]
    Hyper,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "controller source - error parsing config data")]
    Config,
    #[fail(display = "controller source - error converting json data from request")]
    RequestJson,
    #[fail(display = "controller source - error parsing bytes into utf8 from request")]
    RequestUTF8,
    #[fail(display = "controller source - error converting json data from request")]
    ResponseJson,
    #[fail(display = "controller context - error with authentication token")]
    Token,
    #[fail(display = "controller context - missing query despite required params")]
    RequestMissingQuery,
    #[fail(display = "controller context - failed to extract query params")]
    RequestQueryParams,
    #[fail(display = "controller context - error with device id header")]
    DeviceId,
    #[fail(display = "controller context - error with timestamp header")]
    Timestamp,
    #[fail(display = "controller context - error with sign header")]
    Sign,
}

derive_error_impls!();

impl From<ServiceErrorKind> for ErrorKind {
    fn from(err: ServiceErrorKind) -> Self {
        match err {
            ServiceErrorKind::Internal => ErrorKind::Internal,
            ServiceErrorKind::Unauthorized => ErrorKind::Unauthorized,
            ServiceErrorKind::MalformedInput => ErrorKind::BadRequest,
            ServiceErrorKind::NotFound => ErrorKind::NotFound,
            ServiceErrorKind::InvalidInput(validation_errors) => ErrorKind::UnprocessableEntity(validation_errors),
        }
    }
}
