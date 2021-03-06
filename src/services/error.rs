use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use jsonwebtoken::errors::ErrorKind as JwtErrorKind;
use serde_json;
use validator::{ValidationError, ValidationErrors};

use client::storiqa::ErrorKind as StoriqaClientErrorKind;
use client::transactions::ErrorKind as TransactionsClientErrorKind;
use repos::{Error as ReposError, ErrorKind as ReposErrorKind};

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
    InvalidInput(String),
    #[fail(display = "service error - internal error")]
    Internal,
    #[fail(display = "service error - not found")]
    NotFound,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorContext {
    #[fail(display = "service error context - internal error")]
    Internal,
    #[fail(display = "service error context - error inside json web token crate")]
    JsonWebToken,
    #[fail(display = "service error context - invalid token")]
    InvalidToken,
    #[fail(display = "service error context - no account found")]
    NoAccount,
    #[fail(display = "service error context - no user found")]
    NoUser,
    #[fail(display = "service error context - invalid utf8 bytes")]
    UTF8,
    #[fail(display = "service error context - failed to parse string to json")]
    Json,
    #[fail(display = "service error context - rabbit lapin lib")]
    Lapin,
    #[fail(display = "service error context - device already added to user")]
    DeviceAlreadyExists,
    #[fail(display = "service error context - email sending timeout")]
    EmailSending,
    #[fail(display = "service error context - device not added to user")]
    DeviceNotExists,
    #[fail(display = "service error context - received timestamp is less or equal to timestamp in db")]
    WrongTimestamp,
    #[fail(display = "service error context - public key has wrong format")]
    PublicKey,
    #[fail(display = "service error context - can not form message to verify sign")]
    WrongMessage,
    #[fail(display = "service error context - can not form sign")]
    Sign,
    #[fail(display = "service error context - can not verify sign")]
    VerifySign,
    #[fail(display = "service error context - can not load template")]
    NoTemplate,
    #[fail(display = "service error context - render email template")]
    RenderTemplate,
}

derive_error_impls!();

impl From<ReposError> for Error {
    fn from(e: ReposError) -> Error {
        let kind: ErrorKind = e.kind().into();
        e.context(kind).into()
    }
}

impl From<ReposErrorKind> for ErrorKind {
    fn from(e: ReposErrorKind) -> ErrorKind {
        match e {
            ReposErrorKind::Internal => ErrorKind::Internal,
            ReposErrorKind::Unauthorized => ErrorKind::Unauthorized,
            ReposErrorKind::Constraints(validation_errors) => {
                ErrorKind::InvalidInput(serde_json::to_string(&validation_errors).unwrap_or_default())
            }
        }
    }
}

impl From<StoriqaClientErrorKind> for ErrorKind {
    fn from(err: StoriqaClientErrorKind) -> Self {
        match err {
            StoriqaClientErrorKind::Internal => ErrorKind::Internal,
            StoriqaClientErrorKind::Unauthorized => ErrorKind::Unauthorized,
            StoriqaClientErrorKind::MalformedInput => ErrorKind::Internal,
            StoriqaClientErrorKind::Validation(s) => ErrorKind::InvalidInput(s.to_string()),
        }
    }
}

impl From<TransactionsClientErrorKind> for ErrorKind {
    fn from(err: TransactionsClientErrorKind) -> Self {
        match err {
            TransactionsClientErrorKind::Internal => ErrorKind::Internal,
            TransactionsClientErrorKind::Unauthorized => ErrorKind::Internal,
            TransactionsClientErrorKind::MalformedInput => ErrorKind::Internal,
            TransactionsClientErrorKind::Validation(s) => ErrorKind::InvalidInput(s),
        }
    }
}

impl<'a> From<&'a JwtErrorKind> for ErrorKind {
    fn from(err: &JwtErrorKind) -> Self {
        match err {
            JwtErrorKind::ExpiredSignature => {
                let mut errors = ValidationErrors::new();
                let mut error = ValidationError::new("expired");
                error.message = Some("JWT has expired.".into());
                errors.add("token", error);
                ErrorKind::InvalidInput(serde_json::to_string(&errors).unwrap_or_default())
            }
            _ => ErrorKind::Unauthorized,
        }
    }
}
