use std::fmt::{self, Display};
use std::sync::Arc;

use futures::prelude::*;
use hyper::{header::HeaderValue, header::AUTHORIZATION, Body, HeaderMap, Method, Response, Uri};

use super::error::*;
use config::Config;
use models::*;
use prelude::*;
use services::{AccountsService, AuthService, TransactionsService, UsersService};

mod accounts;
mod fallback;
mod transactions;
mod users;

pub use self::accounts::*;
pub use self::fallback::*;
pub use self::transactions::*;
pub use self::users::*;

pub type ControllerFuture = Box<Future<Item = Response<Body>, Error = Error> + Send>;

#[derive(Clone)]
pub struct Context {
    pub body: Vec<u8>,
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap<HeaderValue>,
    pub users_service: Arc<dyn UsersService>,
    pub accounts_service: Arc<dyn AccountsService>,
    pub transactions_service: Arc<dyn TransactionsService>,
    pub auth_service: Arc<dyn AuthService>,
    pub config: Arc<Config>,
}

impl Context {
    pub fn get_auth_token(&self) -> Option<StoriqaJWT> {
        self.headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                let segments = header.split(' ').collect::<Vec<_>>();
                match segments.as_slice() {
                    ["Bearer", t] => Some(StoriqaJWT::new(t.to_string())),
                    _ => None,
                }
            })
    }
    pub fn get_auth_info(&self) -> Result<AuthInfo, Error> {
        let timestamp_header = self
            .headers
            .get("Timestamp")
            .ok_or(ectx!(try err ErrorContext::Timestamp, ErrorKind::Unauthorized))?;
        let timestamp_str = timestamp_header
            .to_str()
            .map_err(|_| ectx!(try err ErrorContext::Timestamp, ErrorKind::Unauthorized))?;
        let timestamp = timestamp_str
            .parse::<i64>()
            .map_err(|_| ectx!(try err ErrorContext::Timestamp, ErrorKind::Unauthorized))?;

        let device_id_header = self
            .headers
            .get("Device-id")
            .ok_or(ectx!(try err ErrorContext::DeviceId, ErrorKind::Unauthorized))?;
        let device_id_str = device_id_header
            .to_str()
            .map_err(|_| ectx!(try err ErrorContext::DeviceId, ErrorKind::Unauthorized))?;
        let device_id = DeviceId::new(device_id_str.to_string());

        let sign_header = self
            .headers
            .get("Sign")
            .ok_or(ectx!(try err ErrorContext::Sign, ErrorKind::Unauthorized))?;
        let sign_str = sign_header
            .to_str()
            .map_err(|_| ectx!(try err ErrorContext::Sign, ErrorKind::Unauthorized))?;

        Ok(AuthInfo::new(timestamp, device_id, sign_str.to_string()))
    }

    pub fn get_auth(&self) -> Result<(AuthInfo, UserId, u64), Error> {
        let token = self
            .get_auth_token()
            .ok_or_else(|| ectx!(try err ErrorContext::Token, ErrorKind::Unauthorized))?;
        let token_clone = token.clone();
        let user_id = self
            .auth_service
            .get_jwt_auth(token.clone())
            .map_err(ectx!(try convert => token))
            .map(|auth| auth.user_id)?;
        let exp = self
            .auth_service
            .get_exp(token_clone.clone())
            .map_err(ectx!(try convert => token_clone))?;

        self.get_auth_info().map(|auth_info| (auth_info, user_id, exp))
    }

    pub fn authenticate(&self) -> impl Future<Item = UserId, Error = Error> + Send {
        let auth_service = self.auth_service.clone();
        self.get_auth().into_future().and_then(move |(auth_info, user_id, exp)| {
            auth_service
                .authenticate(auth_info.clone(), user_id, exp)
                .map_err(ectx!(convert => auth_info, user_id))
                .map(move |_| user_id)
        })
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!(
            "{} {}, headers: {:#?}, body: {:?}",
            self.method,
            self.uri,
            self.headers,
            String::from_utf8(self.body.clone()).ok()
        ))
    }
}
