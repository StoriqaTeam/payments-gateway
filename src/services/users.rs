use super::error::*;
use client::StoriqaClient;
use models::*;
use prelude::*;
use std::sync::Arc;

pub trait UsersService: Send + Sync + 'static {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn create_user(
        &self,
        email: String,
        password: Password,
        first_name: String,
        last_name: String,
    ) -> Box<Future<Item = User, Error = Error> + Send>;
    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn me(&self) -> Box<Future<Item = User, Error = Error> + Send>;
}

pub struct UsersServiceImpl {
    auth: Result<Auth, String>,
    storiqa_client: Arc<dyn StoriqaClient>,
}

impl UsersServiceImpl {
    pub fn new(auth: Result<Auth, String>, storiqa_client: Arc<dyn StoriqaClient>) -> Self {
        UsersServiceImpl { auth, storiqa_client }
    }
}

impl UsersService for UsersServiceImpl {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(self.storiqa_client.get_jwt(email, password).map_err(ectx!(catch)))
    }

    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(
            self.storiqa_client
                .get_jwt_by_oauth(oauth_token, oauth_provider)
                .map_err(ectx!(catch)),
        )
    }

    fn create_user(
        &self,
        email: String,
        password: Password,
        first_name: String,
        last_name: String,
    ) -> Box<Future<Item = User, Error = Error> + Send> {
        Box::new(
            self.storiqa_client
                .create_user(email, password, first_name, last_name)
                .map_err(ectx!(catch)),
        )
    }

    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(self.storiqa_client.confirm_email(token).map_err(ectx!(catch)))
    }

    fn me(&self) -> Box<Future<Item = User, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        let auth = self.auth.clone();
        Box::new(
            auth.map_err(|e| {
                let e = format_err!("{}", e);
                ectx!(err e, ErrorKind::Unauthorized)
            }).into_future()
            .and_then(move |auth| cli.me(auth.token).map_err(ectx!(catch))),
        )
    }
}
