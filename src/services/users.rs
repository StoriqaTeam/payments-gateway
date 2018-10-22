use std::sync::Arc;

use validator::Validate;
use serde_json;

use super::auth::AuthService;
use super::error::*;
use client::StoriqaClient;
use models::*;
use prelude::*;

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
    fn me(&self, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error> + Send>;
}

pub struct UsersServiceImpl {
    auth_service: Arc<dyn AuthService>,
    storiqa_client: Arc<dyn StoriqaClient>,
}

impl UsersServiceImpl {
    pub fn new(auth_service: Arc<dyn AuthService>, storiqa_client: Arc<dyn StoriqaClient>) -> Self {
        UsersServiceImpl {
            auth_service,
            storiqa_client,
        }
    }
}

impl UsersService for UsersServiceImpl {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(self.storiqa_client.get_jwt(email, password).map_err(ectx!(convert)))
    }

    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(
            self.storiqa_client
                .get_jwt_by_oauth(oauth_token, oauth_provider)
                .map_err(ectx!(convert)),
        )
    }

    fn create_user(
        &self,
        email: String,
        password: Password,
        first_name: String,
        last_name: String,
    ) -> Box<Future<Item = User, Error = Error> + Send> {
        let new_user = NewUser::new(email, first_name, last_name, password);
        let client = self.storiqa_client.clone();
        Box::new(
            new_user
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_string(&e).unwrap_or(e.to_string())) => new_user))
                .into_future()
                .and_then(move |_| client.create_user(new_user).map_err(ectx!(convert))),
        )
    }

    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(self.storiqa_client.confirm_email(token).map_err(ectx!(convert)))
    }

    fn me(&self, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(
            self.auth_service
                .authenticate(token)
                .and_then(move |auth| cli.me(auth.token).map_err(ectx!(convert))),
        )
    }
}
