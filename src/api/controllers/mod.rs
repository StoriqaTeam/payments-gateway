use super::auth::Authenticator;
use super::error::*;
use client::{HttpClient, StoriqaClient};
use failure::Fail;
use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};
use models::Auth;
use std::sync::Arc;

mod fallback;
mod users;

pub use self::fallback::*;
pub use self::users::*;

pub type ControllerFuture = Box<Future<Item = Response<Body>, Error = Error> + Send>;

#[derive(Clone)]
pub struct Context {
    pub body: Vec<u8>,
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap<HeaderValue>,
    pub authenticator: Arc<Authenticator>,
    pub client: Arc<HttpClient>,
    pub storiqa_client: Arc<StoriqaClient>,
}

fn authorize(ctx: &Context) -> impl Future<Item = Auth, Error = Error> {
    let headers = ctx.headers.clone();
    ctx.authenticator
        .authenticate(&ctx.headers)
        .map_err(ewrap!(ErrorSource::JwtAuth, ErrorKind::Unauthorized, headers))
        .into_future()
}
