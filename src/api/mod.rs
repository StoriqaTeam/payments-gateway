use hyper;
use hyper::{service::Service, Body, Request, Response};

use super::config::Config;
use super::utils::log_error;
use client::{Client, ClientImpl, StoriqaClient, StoriqaClientImpl};
use failure::{Compat, Fail};
use futures::future;
use futures::prelude::*;
use hyper::Client as HyperClient;
use hyper::Server;
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;
use std::sync::Arc;
use utils::read_body;

mod controllers;
mod error;
mod requests;
mod responses;

use self::controllers::*;
use self::error::{Error, ErrorKind};

#[derive(Clone)]
pub struct ApiService {
    client: Arc<dyn Client>,
    storiqa_client: Arc<dyn StoriqaClient>,
}

impl ApiService {
    fn new(config: &Config) -> Self {
        let client = ClientImpl::new(config);
        let storiqa_client = StoriqaClientImpl::new(&config, client.clone());
        ApiService {
            client: Arc::new(client),
            storiqa_client: Arc::new(storiqa_client),
        }
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
                .map_err(|e| error_context!(e, ErrorKind::Hyper))
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
                        _ => post_sessions,
                    };

                    router(ctx, parts.method.into(), parts.uri.path())
                }).map_err(|e| e.compat()),
        )
    }
}

pub fn start_server(config: Config) {
    hyper::rt::run(future::lazy(move || {
        let app = ApiService::new(&config);
        let new_service = move || {
            let res: Result<_, hyper::Error> = Ok(app.clone());
            res
        };
        format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(|e| error_context!(e, ErrorKind::Parse, config.server.host, config.server.port))
            .into_future()
            .and_then(move |addr| {
                let server = Server::bind(&addr)
                    .serve(new_service)
                    .map_err(move |e| error_context!(e, ErrorKind::Parse, addr));
                info!("Listening on http://{}", addr);
                server
            }).map_err(|e: Error| log_error(e))
    }));
}
