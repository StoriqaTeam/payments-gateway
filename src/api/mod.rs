use hyper;
use hyper::{service::Service, Body, Request, Response};

use super::config::Config;
use super::utils::log_error;
use failure::Fail;
use futures::future;
use futures::future::Either;
use futures::prelude::*;
use hyper::Server;
use std::net::SocketAddr;

mod error;

use self::error::{Error, ErrorKind};

#[derive(Clone)]
pub struct ApiService;

impl Service for ApiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        unimplemented!()
    }
}

pub fn start_server(config: Config) {
    hyper::rt::run(future::lazy(move || {
        let app = ApiService {};
        let new_service = move || {
            let res: Result<_, hyper::Error> = Ok(app.clone());
            res
        };
        format!("{}:{}", config.server.host, config.server.port)
            .parse::<SocketAddr>()
            .map_err(|e| error_context!(e, ErrorKind::Parse, config))
            .into_future()
            .and_then(move |addr| {
                Server::bind(&addr)
                    .serve(new_service)
                    .map(move |_| {
                        info!("Listening on http://{}", addr);
                    })
                    .map_err(move |e| error_context!(e, ErrorKind::Parse, config))
            })
            .map_err(|e: Error| log_error(e))
    }));
}
