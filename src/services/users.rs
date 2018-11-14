use std::sync::Arc;

use serde_json;
use validator::Validate;

use super::auth::AuthService;
use super::error::*;
use client::StoriqaClient;
use models::*;
use prelude::*;
use repos::{DbExecutor, UsersRepo};

pub trait UsersService: Send + Sync + 'static {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn create_user(&self, new_user: NewUser) -> Box<Future<Item = User, Error = Error> + Send>;
    fn update_user(&self, update_user: UpdateUser, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error> + Send>;
    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn reset_password(&self, reset: ResetPassword) -> Box<Future<Item = (), Error = Error> + Send>;
    fn change_password(&self, change_password: ChangePassword, token: AuthenticationToken) -> Box<Future<Item = (), Error = Error> + Send>;
    fn confirm_reset_password(&self, reset: ResetPasswordConfirm) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn me(&self, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error> + Send>;
}

pub struct UsersServiceImpl<E: DbExecutor> {
    auth_service: Arc<dyn AuthService>,
    storiqa_client: Arc<dyn StoriqaClient>,
    users_repo: Arc<dyn UsersRepo>,
    db_executor: E,
}

impl<E: DbExecutor> UsersServiceImpl<E> {
    pub fn new(
        auth_service: Arc<dyn AuthService>,
        storiqa_client: Arc<dyn StoriqaClient>,
        users_repo: Arc<dyn UsersRepo>,
        db_executor: E,
    ) -> Self {
        UsersServiceImpl {
            auth_service,
            storiqa_client,
            users_repo,
            db_executor,
        }
    }
}

impl<E: DbExecutor> UsersService for UsersServiceImpl<E> {
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

    fn create_user(&self, new_user: NewUser) -> Box<Future<Item = User, Error = Error> + Send> {
        let client = self.storiqa_client.clone();
        let users_repo = self.users_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(
            new_user
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_value(&e).unwrap_or_default()) => new_user))
                .into_future()
                .and_then(move |_| client.create_user(new_user).map_err(ectx!(convert)))
                .and_then(move |user| {
                    db_executor.execute(move || {
                        let user_db: NewUserDB = user.clone().into();
                        users_repo.create(user_db.clone()).map_err(ectx!(try convert => user_db))?;
                        Ok(user)
                    })
                }),
        )
    }

    fn update_user(&self, update_user: UpdateUser, token: AuthenticationToken) -> Box<Future<Item = User, Error = Error> + Send> {
        let client = self.storiqa_client.clone();
        let users_repo = self.users_repo.clone();
        let auth_service = self.auth_service.clone();
        let db_executor = self.db_executor.clone();
        let update_user_clone = update_user.clone();
        let update_user_clone2 = update_user.clone();
        Box::new(
            update_user
                .validate()
                .map_err(
                    |e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_value(&e).unwrap_or_default()) => update_user_clone2),
                ).into_future()
                .and_then(move |_| auth_service.authenticate(token))
                .and_then(move |auth| client.update_user(update_user, auth.user_id).map_err(ectx!(convert)))
                .and_then(move |user| {
                    db_executor.execute(move || {
                        users_repo
                            .update(user.id, update_user_clone.clone())
                            .map_err(ectx!(try convert => update_user_clone))?;
                        Ok(user)
                    })
                }),
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
    fn reset_password(&self, reset: ResetPassword) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(self.storiqa_client.reset_password(reset).map_err(ectx!(convert)))
    }
    fn change_password(&self, change_password: ChangePassword, token: AuthenticationToken) -> Box<Future<Item = (), Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(
            self.auth_service
                .authenticate(token)
                .and_then(move |auth| cli.change_password(change_password, auth.token).map_err(ectx!(convert))),
        )
    }
    fn confirm_reset_password(&self, confirm: ResetPasswordConfirm) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(
            confirm
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_value(&e).unwrap_or_default()) => confirm))
                .into_future()
                .and_then(move |_| cli.confirm_reset_password(confirm).map_err(ectx!(convert))),
        )
    }
}
