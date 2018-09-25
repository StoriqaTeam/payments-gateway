use hyper;
use hyper::{service::Service, Body, Request, Response};

use super::config::Config;
use super::utils::log_error;
use failure::{Compat, Fail};
use futures::future;
use futures::prelude::*;
use hyper::Server;
use std::net::SocketAddr;

mod controllers;
mod error;

use self::controllers::*;
use self::error::{Error, ErrorKind};

#[derive(Clone)]
pub struct ApiService;

impl Service for ApiService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, http_body) = req.into_parts();
        Box::new(
            read_body(http_body)
                .and_then(move |body| {
                    let ctx = Context {
                        body,
                        method: parts.method.clone(),
                        uri: parts.uri.clone(),
                        headers: parts.headers,
                    };
                    let router = router! {
                        _ => post_sessions,
                    };

                    router(ctx, parts.method.into(), parts.uri.path())
                })
                .map_err(|e| e.compat()),
        )
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

// Reads body of request in Future format
pub fn read_body(body: hyper::Body) -> impl Future<Item = Vec<u8>, Error = Error> {
    body.fold(Vec::new(), |mut acc, chunk| {
        acc.extend_from_slice(&*chunk);
        future::ok::<_, hyper::Error>(acc)
    }).map_err(|e| error_context!(e, ErrorKind::Hyper))
}
