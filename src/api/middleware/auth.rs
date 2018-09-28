use super::super::error::Error;
use hyper::header::{Authorization, Bearer};
use hyper::{service::Service, Body, Request, Response};
use jsonwebtoken::{decode, Algorithm, Validation};
use models::*;

use failure::{Compat, Fail};
use futures::prelude::*;

#[derive(Clone)]
pub struct AuthMiddleware<T>
where
    T: Clone,
{
    upstream: T,
    jwt_public_key: Vec<u8>,
    jwt_valid_secs: usize,
}

impl<T> AuthMiddleware<T>
where
    T: Clone,
{
    pub fn new(upstream: T, jwt_public_key: Vec<u8>, jwt_valid_secs: usize) -> Self {
        Self {
            upstream,
            jwt_public_key,
            jwt_valid_secs,
        }
    }
}

impl<T> Service for AuthMiddleware<T>
where
    T: Service<
            ReqBody = Body,
            ResBody = Body,
            Error = Compat<Error>,
            Future = Box<Future<Item = Response<Body>, Error = Compat<Error>> + Send>,
        > + Clone,
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<Bearer>>();

        let token_payload = auth_header.and_then(move |auth| {
            let token = auth.0.token.as_ref();
            decode::<JWTClaims>(token, &self.jwt_public_key, &validation).ok().map(|t| t.claims)
        });
        self.upstream.call(req)
    }
}
