use jsonwebtoken::{decode, Algorithm, Validation};

use super::error::*;
use super::ServiceFuture;
use models::*;
use prelude::*;

pub trait AuthService: Send + Sync + 'static {
    fn authenticate(&self, token: AuthenticationToken) -> ServiceFuture<Auth>;
    fn get_exp(&self, token: AuthenticationToken) -> ServiceFuture<u64>;
}

pub struct AuthServiceImpl {
    jwt_public_key: Vec<u8>,
    jwt_valid_secs: usize,
}

impl AuthServiceImpl {
    pub fn new(jwt_public_key: Vec<u8>, jwt_valid_secs: usize) -> Self {
        Self {
            jwt_public_key,
            jwt_valid_secs,
        }
    }
}

impl AuthService for AuthServiceImpl {
    fn authenticate(&self, token: AuthenticationToken) -> ServiceFuture<Auth> {
        let validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        let token_clone = token.clone();
        Box::new(
            decode::<JWTClaims>(token_clone.raw(), &self.jwt_public_key, &validation)
                .map_err(ectx!(ErrorContext::JsonWebToken, ErrorKind::Unauthorized => token_clone.raw()))
                .map(move |t| Auth {
                    user_id: t.claims.user_id,
                    token: StoriqaJWT::new(token.raw().to_string()),
                }).into_future(),
        )
    }
    fn get_exp(&self, token: AuthenticationToken) -> ServiceFuture<u64> {
        let token_clone = token.clone();
        let validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        Box::new(
            decode::<JWTClaims>(token_clone.raw(), &self.jwt_public_key, &validation)
                .map_err(ectx!(ErrorContext::JsonWebToken, ErrorKind::Unauthorized => token_clone.raw()))
                .map(move |t| t.claims.exp)
                .into_future(),
        )
    }
}
