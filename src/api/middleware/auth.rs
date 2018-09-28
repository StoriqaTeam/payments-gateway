use super::super::error::Error;
use hyper::{service::Service, Body, Request, Response};

use failure::{Compat, Fail};
use futures::prelude::*;

struct AuthMiddleware<T> {
    upstream: T,
    jwt_public_key: Vec<u8>,
}

impl<T> Service for AuthMiddleware<T>
where
    T: Service<
        ReqBody = Body,
        ResBody = Body,
        Error = Compat<Error>,
        Future = Box<Future<Item = Response<Body>, Error = Compat<Error>> + Send>,
    >,
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.upstream.call(req)
    }
}
