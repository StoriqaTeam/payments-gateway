use super::error::*;
use client::{HttpClient, StoriqaClient};
use failure::Fail;
use futures::prelude::*;
use hyper::{header::HeaderValue, Body, HeaderMap, Method, Response, Uri};
use serde::Serialize;
use serde_json;
use std::fmt::Debug;
use std::sync::Arc;

mod auth;
pub use self::auth::*;

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

pub fn response_with_model<M>(model: &M) -> ControllerFuture
where
    M: Debug + Serialize,
{
    Box::new(
        serde_json::to_string(&model)
            .map_err(ewrap!(ErrorSource::ResponseJson, ErrorKind::Internal, model))
            .into_future()
            .map(|text| {
                Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(text.into())
                    .unwrap()
            }),
    )
}
