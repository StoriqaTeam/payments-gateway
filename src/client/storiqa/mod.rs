mod responses;

use self::responses::*;
use super::Client;
use super::{Error, ErrorKind};
use config::Config;
use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use models::*;
use serde_json;
use std::sync::Arc;
use utils::read_body;

pub trait StoriqaClient: Send + Sync + 'static {
    fn getJWT(&self, email: String, password: String) -> Box<Future<Item = StoriqaJWT, Error = Error>>;
}

pub struct StoriqaClientImpl {
    cli: Arc<Client>,
    storiqa_url: String,
}

impl StoriqaClientImpl {
    fn new<C: Client>(config: &Config, cli: C) -> Self {
        Self {
            cli: Arc::new(cli),
            storiqa_url: config.client.storiqa_url.clone(),
        }
    }
}

impl StoriqaClient for StoriqaClientImpl {
    fn getJWT(&self, email: String, password: String) -> Box<Future<Item = StoriqaJWT, Error = Error>> {
        let query = format!(
            r#"
                mutation M {{
                    getJWTByEmail(input: {{email: "{}", password: "{}"}}) {{
                        token
                    }}
                }}
            "#,
            email, password
        );
        let query1 = query.clone();
        let query2 = query.clone();
        let query3 = query.clone();
        let cli = self.cli.clone();
        Box::new(
            Request::builder()
                .method(Method::POST)
                .body(Body::from(query))
                .map_err(move |e| error_context!(e, ErrorKind::Http, query3))
                .into_future()
                .and_then(move |req| cli.request(req).map_err(move |e| error_context!(e, ErrorKind::Hyper, query1)))
                .and_then(move |resp| read_body(resp.into_body()).map_err(move |e| error_context!(e, ErrorKind::Hyper, query2)))
                .and_then(|bytes| {
                    let bytes_clone = bytes.clone();
                    String::from_utf8(bytes).map_err(move |e| error_context!(e, ErrorKind::UTF8, bytes_clone))
                })
                .and_then(|string| serde_json::from_str::<GetJWTResponse>(&string).map_err(|e| error_context!(e, ErrorKind::Json, string)))
                .and_then(|resp| resp.data.ok_or(ErrorKind::Unauthorized.into()))
                .map(|resp_data| StoriqaJWT::new(resp_data.getJWTByEmail.token)),
        )
    }
}

const getJWTQuery: &'static str = r#"
mutation M {{
	getJWTByEmail(input: {{email: "{}", password: "{}"}}) {{
        token
    }}
}}
"#;
