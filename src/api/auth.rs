use hyper::header::{HeaderMap, AUTHORIZATION};
use jsonwebtoken::{decode, Algorithm, Validation};
use models::*;

pub trait Authenticator: Send + Sync + 'static {
    fn authenticate(&self, headers: &HeaderMap) -> Option<Auth>;
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
    fn authenticate(&self, headers: &HeaderMap) -> Option<Auth> {
        let validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                let len = "Bearer: ".len();
                if header.len() > len {
                    Some(header[len..].to_string())
                } else {
                    None
                }
            }).and_then(|token| {
                decode::<JWTClaims>(&token, &self.jwt_public_key, &validation)
                    .ok()
                    .map(move |t| Auth {
                        user_id: t.claims.user_id,
                        token: StoriqaJWT::new(token),
                    })
            })
    }
}
