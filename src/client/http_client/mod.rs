mod error;

use self::error::*;
use config::Config;
use failure::Fail;
use futures::prelude::*;
use hyper;
use hyper::{client::HttpConnector, Body, Request, Response};
use hyper_tls::HttpsConnector;

pub trait HttpClient: Send + Sync + 'static {
    fn request(&self, req: Request<Body>) -> Box<Future<Item = Response<Body>, Error = Error> + Send>;
}

#[derive(Clone)]
pub struct HttpClientImpl {
    cli: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl HttpClientImpl {
    pub fn new(config: &Config) -> Self {
        let mut connector = HttpsConnector::new(config.client.dns_threads).unwrap();
        connector.https_only(true);
        let cli = hyper::Client::builder().build(connector);
        Self { cli }
    }
}

impl HttpClient for HttpClientImpl {
    fn request(&self, req: Request<Body>) -> Box<Future<Item = Response<Body>, Error = Error> + Send> {
        Box::new(
            self.cli
                .request(req)
                .map_err(ewrap!(ErrorSource::Hyper, ErrorKind::Internal))
                .and_then(|resp| {
                    if resp.status().is_client_error() || resp.status().is_server_error() {
                        match resp.status().as_u16() {
                            400 => Err(ewrap!(err ErrorSource::Server, ErrorKind::BadRequest)),
                            401 => Err(ewrap!(err ErrorSource::Server, ErrorKind::Unauthorized)),
                            404 => Err(ewrap!(err ErrorSource::Server, ErrorKind::NotFound)),
                            500 => Err(ewrap!(err ErrorSource::Server, ErrorKind::Internal)),
                            502 => Err(ewrap!(err ErrorSource::Server, ErrorKind::BadGateway)),
                            504 => Err(ewrap!(err ErrorSource::Server, ErrorKind::GatewayTimeout)),
                            _ => Err(ewrap!(err ErrorSource::Server, ErrorKind::UnknownServerError)),
                        }
                    } else {
                        Ok(resp)
                    }
                }),
        )
    }
}
