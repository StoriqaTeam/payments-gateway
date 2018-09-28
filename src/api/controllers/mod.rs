use super::error::*;
use client::{HttpClient, StoriqaClient};
use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};
use std::sync::Arc;

mod auth;
mod fallback;
pub use self::auth::*;
pub use self::fallback::*;

pub type ControllerFuture = Box<Future<Item = Response<Body>, Error = Error> + Send>;

#[derive(Clone)]
pub struct Context {
    pub body: Vec<u8>,
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap<HeaderValue>,
    pub client: Arc<HttpClient>,
    pub storiqa_client: Arc<StoriqaClient>,
}
