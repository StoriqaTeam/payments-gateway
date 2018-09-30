use super::error::*;
use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};
use models::Auth;
use services::UsersService;
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
    pub auth: Result<Auth, String>,
    pub users_service: Arc<dyn UsersService>,
}

// fn authenticate(ctx: &Context) -> impl Future<Item = Auth, Error = Error> {
//     let headers = ctx.headers.clone();
//     ctx.authenticator
//         .authenticate(&ctx.headers)
//         .map_err(ectx!(ErrorKind::Unauthorized => headers))
//         .into_future()
// }
