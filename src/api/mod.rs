use hyper;
use hyper::{service::Service, Body, Request, Response};

use super::config::Config;
use super::utils::{log_error, log_warn};
use base64;
use client::{HttpClient, HttpClientImpl, StoriqaClient, StoriqaClientImpl};
use failure::{Compat, Fail};
use futures::future;
use futures::prelude::*;
use hyper::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use utils::read_body;

mod controllers;
mod error;
mod middleware;
mod requests;
mod responses;
mod utils;

use self::controllers::*;
use self::error::*;

#[derive(Clone)]
pub struct ApiService {
    client: Arc<dyn HttpClient>,
    storiqa_client: Arc<dyn StoriqaClient>,
    storiqa_jwt_public_key: Vec<u8>,
}

impl ApiService {
    fn from_config(config: &Config) -> Result<Self, Error> {
        let client = HttpClientImpl::new(config);
        let storiqa_client = StoriqaClientImpl::new(&config, client.clone());
        let storiqa_jwt_public_key: Result<Vec<u8>, Error> = base64::decode(&config.client.storiqa_jwt_public_key_base64).map_err(ewrap!(
            ErrorSource::Config,
            ErrorKind::Internal,
            &config.client.storiqa_jwt_public_key_base64
        ));
        let storiqa_jwt_public_key = storiqa_jwt_public_key?;
        Ok(ApiService {
            client: Arc::new(client),
            storiqa_client: Arc::new(storiqa_client),
            storiqa_jwt_public_key,
        })
    }
}

impl Service for ApiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, http_body) = req.into_parts();
        let client = self.client.clone();
        let storiqa_client = self.storiqa_client.clone();
        Box::new(
            read_body(http_body)
                .map_err(ewrap!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(move |body| {
                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                        client,
                        storiqa_client,
                    };
                    let router = router! {
                        POST /v1/sessions => post_sessions,
                        POST /v1/users => post_users,
                        _ => not_found,
                    };

                    router(ctx, parts.method.into(), parts.uri.path())
                })
                .or_else(|e| match e.kind() {
                    ErrorKind::BadRequest => {
                        log_error(&e);
                        Ok(Response::builder()
                            .status(400)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Bad request"}"#))
                            .unwrap())
                    }
                    ErrorKind::Unauthorized => {
                        log_warn(&e);
                        Ok(Response::builder()
                            .status(401)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Unauthorized"}"#))
                            .unwrap())
                    }
                    ErrorKind::Internal => {
                        log_error(&e);
                        Ok(Response::builder()
                            .status(500)
                            .header("Content-Type", "application/json")
                            .body(Body::from(r#"{"description": "Internal server error"}"#))
                            .unwrap())
                    }
                }),
        )
    }
}

pub fn start_server(config: Config) {
    hyper::rt::run(future::lazy(move || {
        let config_clone = config.clone();
        format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(ewrap!(
                ErrorSource::Config,
                ErrorKind::Internal,
                config.server.host,
                config.server.port
            ))
            .and_then(move |addr| ApiService::from_config(&config_clone).map(move |app| (app, addr)))
            .into_future()
            .and_then(move |(app, addr)| {
                let new_service = move || {
                    let res: Result<_, hyper::Error> = Ok(app.clone());
                    res
                };
                let server = Server::bind(&addr)
                    .serve(new_service)
                    .map_err(ewrap!(ErrorSource::Hyper, ErrorKind::Internal, addr));
                info!("Listening on http://{}", addr);
                server
            })
            .map_err(|e: Error| log_error(&e))
    }));
}
