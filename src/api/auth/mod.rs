mod error;

use self::error::*;
use failure::Fail;
use hyper::header::{HeaderMap, AUTHORIZATION};
use jsonwebtoken::{decode, Algorithm, Validation};
use models::*;

pub trait Authenticator: Send + Sync + 'static {
    fn authenticate(&self, headers: &HeaderMap) -> Result<Auth, Error>;
}

pub struct AuthenticatorImpl {
    jwt_public_key: Vec<u8>,
    jwt_valid_secs: usize,
}

impl AuthenticatorImpl {
    pub fn new(jwt_public_key: Vec<u8>, jwt_valid_secs: usize) -> Self {
        AuthenticatorImpl {
            jwt_public_key,
            jwt_valid_secs,
        }
    }
}

impl Authenticator for AuthenticatorImpl {
    fn authenticate(&self, headers: &HeaderMap) -> Result<Auth, Error> {
        let validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        let headers_clone = headers.clone();
        headers
            .get(AUTHORIZATION)
            .ok_or(ewrap!(err ErrorContext::NoAuthHeader, ErrorKind::Unauthorized => headers_clone))
            .and_then(|header| {
                header
                    .to_str()
                    .map_err(ewrap!(ErrorContext::ParseAuthHeader, ErrorKind::Unauthorized))
            }).and_then(|header| {
                let len = "Bearer ".len();
                if (header.len() > len) && header.starts_with("Bearer ") {
                    Ok(header[len..].to_string())
                } else {
                    Err(ewrap!(err ErrorContext::InvalidBearer, ErrorKind::Unauthorized => header))
                }
            }).and_then(|token: String| {
                let token_clone = token.clone();
                decode::<JWTClaims>(&token, &self.jwt_public_key, &validation)
                    .map_err(ewrap!(ErrorContext::JsonWebToken, ErrorKind::Unauthorized => token))
                    .map(move |t| Auth {
                        user_id: t.claims.user_id,
                        token: StoriqaJWT::new(token_clone),
                    })
            })
    }
}
