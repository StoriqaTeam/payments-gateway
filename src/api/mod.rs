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

mod auth;
mod controllers;
mod error;
mod requests;
mod responses;
mod utils;

use self::auth::{Authenticator, AuthenticatorImpl};
use self::controllers::*;
use self::error::*;

#[derive(Clone)]
pub struct ApiService {
    client: Arc<dyn HttpClient>,
    authenticator: Arc<dyn Authenticator>,
    storiqa_client: Arc<dyn StoriqaClient>,
    server_address: SocketAddr,
    config: Config,
}

impl ApiService {
    fn from_config(config: &Config) -> Result<Self, Error> {
        let client = HttpClientImpl::new(config);
        let storiqa_client = StoriqaClientImpl::new(&config, client.clone());
        let storiqa_jwt_public_key_base64 = config.auth.storiqa_jwt_public_key_base64.clone();
        let storiqa_jwt_public_key: Result<Vec<u8>, Error> = base64::decode(&config.auth.storiqa_jwt_public_key_base64).map_err(ectx!(
            ErrorContext::Config,
            ErrorKind::Internal =>
            storiqa_jwt_public_key_base64
        ));
        let storiqa_jwt_public_key = storiqa_jwt_public_key?;
        let server_address: Result<SocketAddr, Error> = format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(ectx!(
                ErrorContext::Config,
                ErrorKind::Internal =>
                config.server.host,
                config.server.port
            ));
        let server_address = server_address?;
        let authenticator = AuthenticatorImpl::new(storiqa_jwt_public_key, config.auth.storiqa_jwt_valid_secs);
        Ok(ApiService {
            client: Arc::new(client),
            storiqa_client: Arc::new(storiqa_client),
            config: config.clone(),
            authenticator: Arc::new(authenticator),
            server_address,
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
        let authenticator = self.authenticator.clone();
        Box::new(
            read_body(http_body)
                .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(move |body| {
                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                        client,
                        storiqa_client,
                        authenticator,
                    };
                    let router = router! {
                        POST /v1/sessions => post_sessions,
                        POST /v1/users => post_users,
                        POST /v1/users/confirm_email => post_users_confirm_email,
                        GET /v1/users/me => get_users_me,
                        _ => not_found,
                    };

                    router(ctx, parts.method.into(), parts.uri.path())
                }).or_else(|e| match e.kind() {
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
        ApiService::from_config(&config)
            .into_future()
            .and_then(move |api| {
                let api_clone = api.clone();
                let new_service = move || {
                    let res: Result<_, hyper::Error> = Ok(api_clone.clone());
                    res
                };
                let addr = api.server_address.clone();
                let server = Server::bind(&api.server_address)
                    .serve(new_service)
                    .map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => addr));
                info!("Listening on http://{}", addr);
                server
            }).map_err(|e: Error| log_error(&e))
    }));
}
