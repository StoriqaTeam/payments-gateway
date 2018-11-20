use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use serde_json;

use client::http_client::error::ErrorKind as HttpClientErrorKind;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "storiqa client error - malformed input")]
    MalformedInput,
    #[fail(display = "storiqa client error - unauthorized")]
    Unauthorized,
    #[fail(display = "storiqa client error - internal error")]
    Internal,
    #[fail(display = "storiqa client error - bad request")]
    Validation(serde_json::Value),
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorSource {
    #[fail(display = "storiqa client source - error inside of Hyper library")]
    Hyper,
    #[fail(display = "storiqa client source - error parsing bytes to utf8")]
    Utf8,
    #[fail(display = "storiqa client source - error parsing string to json")]
    Json,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "storiqa client source - no data returned from graphql")]
    NoGraphQLData,
}

derive_error_impls!();

impl From<HttpClientErrorKind> for ErrorKind {
    fn from(err: HttpClientErrorKind) -> Self {
        match err {
            HttpClientErrorKind::Validation(s) => ErrorKind::Validation(serde_json::to_value(s).unwrap_or_default()),
            _ => ErrorKind::Internal,
        }
    }
}
