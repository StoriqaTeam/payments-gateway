mod error;
mod responses;

pub use self::error::*;
use self::responses::*;
use super::HttpClient;
use config::Config;
use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use models::*;
use serde::Deserialize;
use serde_json;
use std::sync::Arc;
use utils::read_body;

pub trait StoriqaClient: Send + Sync + 'static {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
}

pub struct StoriqaClientImpl {
    cli: Arc<HttpClient>,
    storiqa_url: String,
}

impl StoriqaClientImpl {
    pub fn new<C: HttpClient>(config: &Config, cli: C) -> Self {
        Self {
            cli: Arc::new(cli),
            storiqa_url: config.client.storiqa_url.clone(),
        }
    }

    fn exec_query<T: for<'de> Deserialize<'de> + Send>(&self, query: &str) -> impl Future<Item = T, Error = Error> + Send {
        let query = query.to_string();
        let query1 = query.clone();
        let query2 = query.clone();
        let query3 = query.clone();
        let cli = self.cli.clone();
        let query = query.replace("\n", "");
        let body = format!(
            r#"
                {{
                    "operationName": "M",
                    "query": "{}",
                    "variables": null
                }}
            "#,
            query
        );
        Request::builder()
            .uri(self.storiqa_url.clone())
            .method(Method::POST)
            .body(Body::from(body))
            .map_err(ewrap!(ErrorSource::Hyper, ErrorKind::MalformedInput, query3))
            .into_future()
            .and_then(move |req| {
                cli.request(req)
                    .map_err(ewrap!(ErrorSource::HttpClient, ErrorKind::Internal, query1))
            }).and_then(move |resp| read_body(resp.into_body()).map_err(ewrap!(ErrorSource::Hyper, ErrorKind::Internal, query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ewrap!(ErrorSource::Utf8, ErrorKind::Internal, bytes_clone))
            }).and_then(|string| serde_json::from_str::<T>(&string).map_err(ewrap!(ErrorSource::Json, ErrorKind::Internal, string)))
    }
}

impl StoriqaClient for StoriqaClientImpl {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    getJWTByEmail(input: {{email: \"{}\", password: \"{}\", clientMutationId:\"\"}}) {{
                        token
                    }}
                }}
            "#,
            email,
            password.inner()
        );
        Box::new(
            self.exec_query::<GetJWTResponse>(&query)
                .and_then(|resp| {
                    let e = format_err!("Failed at getJWT");
                    resp.data
                        .clone()
                        .ok_or(ewrap!(raw e, ErrorSource::HttpClient, ErrorKind::Unauthorized, resp))
                }).map(|resp_data| StoriqaJWT::new(resp_data.get_jwt_by_email.token)),
        )
    }
}
