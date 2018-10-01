use super::error::*;
use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};
use models::AuthResult;
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
    pub auth_result: AuthResult,
    pub users_service: Arc<dyn UsersService>,
}
