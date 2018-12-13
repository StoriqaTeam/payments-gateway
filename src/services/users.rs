use std::sync::Arc;

use serde_json;
use validator::{Validate, ValidationError, ValidationErrors};

use super::error::*;
use client::StoriqaClient;
use models::*;
use prelude::*;
use repos::{DbExecutor, DeviceTokensRepo, DevicesRepo, UsersRepo};
use services::EmailSenderService;

pub trait UsersService: Send + Sync + 'static {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn merge_user(&self, user: User) -> Box<Future<Item = bool, Error = Error> + Send>;
    fn create_user(&self, new_user: NewUser) -> Box<Future<Item = User, Error = Error> + Send>;
    fn update_user(&self, update_user: UpdateUser, user_id: UserId, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send>;
    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn add_device(
        &self,
        device_id: DeviceId,
        device_os: String,
        public_key: DevicePublicKey,
        user_id: UserId,
    ) -> Box<Future<Item = (), Error = Error> + Send>;
    fn confirm_add_device(&self, token: DeviceConfirmToken, device_id: Option<DeviceId>) -> Box<Future<Item = (), Error = Error> + Send>;
    fn reset_password(&self, reset: ResetPassword) -> Box<Future<Item = (), Error = Error> + Send>;
    fn resend_email_verify(&self, reset: ResendEmailVerify) -> Box<Future<Item = (), Error = Error> + Send>;
    fn change_password(&self, change_password: ChangePassword, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn confirm_reset_password(&self, reset: ResetPasswordConfirm) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn me(&self, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send>;
    fn refresh_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn revoke_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
}

pub struct UsersServiceImpl<E: DbExecutor> {
    storiqa_client: Arc<dyn StoriqaClient>,
    users_repo: Arc<dyn UsersRepo>,
    devices_repo: Arc<dyn DevicesRepo>,
    devices_tokens_repo: Arc<dyn DeviceTokensRepo>,
    db_executor: E,
    email_sender: Arc<dyn EmailSenderService>,
    token_expiration: usize,
    email_sending_timeout: usize,
}

impl<E: DbExecutor> UsersServiceImpl<E> {
    pub fn new(
        storiqa_client: Arc<dyn StoriqaClient>,
        users_repo: Arc<dyn UsersRepo>,
        devices_repo: Arc<dyn DevicesRepo>,
        devices_tokens_repo: Arc<dyn DeviceTokensRepo>,
        db_executor: E,
        email_sender: Arc<dyn EmailSenderService>,
        token_expiration: usize,
        email_sending_timeout: usize,
    ) -> Self {
        UsersServiceImpl {
            storiqa_client,
            users_repo,
            devices_repo,
            devices_tokens_repo,
            db_executor,
            email_sender,
            token_expiration,
            email_sending_timeout,
        }
    }
}

impl<E: DbExecutor> UsersService for UsersServiceImpl<E> {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(
            self.storiqa_client
                .get_jwt(email.clone(), password.clone())
                .map_err(ectx!(convert => email.clone(), password)),
        )
    }

    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(
            self.storiqa_client
                .get_jwt_by_oauth(oauth_token.clone(), oauth_provider.clone())
                .map_err(ectx!(convert => oauth_token, oauth_provider)),
        )
    }

    fn create_user(&self, new_user: NewUser) -> Box<Future<Item = User, Error = Error> + Send> {
        let client = self.storiqa_client.clone();
        let users_repo = self.users_repo.clone();
        let devices_repo = self.devices_repo.clone();
        let db_executor = self.db_executor.clone();
        let new_user_clone = new_user.clone();
        Box::new(
            new_user
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_string(&e).unwrap_or_default()) => new_user))
                .into_future()
                .and_then(move |_| client.create_user(new_user.clone()).map_err(ectx!(convert => new_user)))
                .and_then(move |user| {
                    db_executor.execute_transaction(move || {
                        let user_db: NewUserDB = user.clone().into();
                        users_repo.create(user_db.clone()).map_err(ectx!(try convert => user_db))?;
                        let new_device = NewDevice::new(
                            new_user_clone.device_id,
                            new_user_clone.device_os,
                            user.id,
                            new_user_clone.public_key,
                        );
                        devices_repo.create(new_device.clone()).map_err(ectx!(try convert => new_device))?;
                        Ok(user)
                    })
                }),
        )
    }

    fn merge_user(&self, user: User) -> Box<Future<Item = bool, Error = Error> + Send> {
        let users_repo = self.users_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(db_executor.execute(move || {
            let email = user.email.clone();
            let user_db = users_repo.get_by_email(email.clone()).map_err(ectx!(try convert => email))?;
            if user_db.is_none() {
                users_repo.create(user.clone().into()).map_err(ectx!(try convert => user))?;
                Ok(true)
            } else {
                Ok(false)
            }
        }))
    }

    fn update_user(&self, update_user: UpdateUser, user_id: UserId, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send> {
        let client = self.storiqa_client.clone();
        let users_repo = self.users_repo.clone();
        let db_executor = self.db_executor.clone();
        let update_user_clone = update_user.clone();
        let update_user_clone2 = update_user.clone();
        Box::new(
            update_user
                .validate()
                .map_err(
                    |e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_string(&e).unwrap_or_default()) => update_user_clone2),
                )
                .into_future()
                .and_then(move |_| {
                    client
                        .update_user(update_user.clone(), user_id, token.clone())
                        .map_err(ectx!(convert => update_user, user_id, token))
                })
                .and_then(move |user| {
                    db_executor.execute(move || {
                        let user_id = user.id.clone();
                        users_repo
                            .update(user_id, update_user_clone.clone())
                            .map_err(ectx!(try convert => user_id, update_user_clone))?;
                        Ok(user)
                    })
                }),
        )
    }

    fn add_device(
        &self,
        device_id: DeviceId,
        device_os: String,
        public_key: DevicePublicKey,
        user_id: UserId,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        let devices_repo = self.devices_repo.clone();
        let devices_tokens_repo = self.devices_tokens_repo.clone();
        let db_executor = self.db_executor.clone();
        let email_sending_timeout = self.email_sending_timeout.clone();
        let users_repo = self.users_repo.clone();
        let email_sender = self.email_sender.clone();
        let device_id_clone3 = device_id.clone();
        Box::new(
            db_executor
                .execute(move || {
                    let device_id_clone = device_id.clone();

                    let user = users_repo.get(user_id).map_err(ectx!(try convert => user_id))?;

                    let user = user.ok_or_else(|| ectx!(try err ErrorContext::NoUser, ErrorKind::Unauthorized))?;

                    let device = devices_repo
                        .get(device_id.clone(), user_id)
                        .map_err(ectx!(try convert => device_id, user_id))?;

                    let device_id_clone2 = device_id_clone.clone();
                    if device.is_some() {
                        let mut errors = ValidationErrors::new();
                        let mut error = ValidationError::new("exists");
                        error.add_param("message".into(), &"device already exists".to_string());
                        error.add_param("details".into(), &"no details".to_string());
                        errors.add("device", error);
                        return Err(ectx!(err ErrorContext::DeviceAlreadyExists, ErrorKind::InvalidInput(serde_json::to_string(&errors).unwrap_or_default()) => user_id, device_id_clone2));
                    }

                    let public_key_clone = public_key.clone();

                    let token = devices_tokens_repo.get_by_public_key(public_key_clone.clone()).map_err(ectx!(try convert => public_key_clone))?;

                    if let Some(token) = token {
                        let token_duration = (::chrono::Utc::now().naive_utc() - token.updated_at).num_seconds() as usize;
                        if token_duration < email_sending_timeout  {
                            let mut errors = ValidationErrors::new();
                            let mut error = ValidationError::new("email_timeout");
                            error.add_param("message".into(), &"can not send email more often then 30 seconds".to_string());
                            error.add_param("details".into(), &"no details".to_string());
                            errors.add("email", error);
                            return Err(ectx!(err ErrorContext::EmailSending, ErrorKind::InvalidInput(serde_json::to_string(&errors).unwrap_or_default()) => token));
                        }
                    }

                    let device_id_clone2 = device_id_clone.clone();

                    let new_devices_tokens = NewDeviceToken::new(device_id_clone.clone(), device_os, user_id, public_key);

                    let token = devices_tokens_repo
                        .upsert(new_devices_tokens.clone())
                        .map_err(ectx!(try convert => new_devices_tokens, user_id, device_id_clone2))?;

                    Ok((user, token.id))
                }).and_then(move |(user, token)| {
                    email_sender
                        .send_add_device(user, token, device_id_clone3)
                })
        )
    }

    fn confirm_add_device(
        &self,
        token: DeviceConfirmToken,
        device_id_ex: Option<DeviceId>,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        let db_executor = self.db_executor.clone();
        let devices_repo = self.devices_repo.clone();
        let devices_tokens_repo = self.devices_tokens_repo.clone();
        let token_expiration = self.token_expiration.clone();

        Box::new(db_executor.execute_transaction(move || {

            let device_token = devices_tokens_repo.get(token).map_err(ectx!(try convert => token))?;

            let DeviceToken {
                device_id,
                device_os,
                user_id,
                public_key,
                updated_at,
                ..
            } = device_token.ok_or_else(|| ectx!(try err ErrorContext::InvalidToken, ErrorKind::NotFound => token))?;

            if let Some(device_id_ex) = device_id_ex {
                if device_id_ex != device_id {
                    let mut errors = ValidationErrors::new();
                    let mut error = ValidationError::new("device_id");
                    error.message = Some("device confirmed differs from your device".into());
                    errors.add("device", error);
                    return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::InvalidInput(serde_json::to_string(&errors).unwrap_or_default()) => device_id_ex, device_id));
                }
            }

            let device_id_clone = device_id.clone();
            let device = devices_repo.get(device_id_clone.clone(), user_id)
                .map_err(ectx!(try convert => device_id_clone, user_id))?;

            // if user wants to confirm his device again and again he will receive Ok(()) everytime
            if device.is_none() {
                let token_duration = (::chrono::Utc::now().naive_utc() - updated_at).num_seconds() as usize;
                if token_duration > token_expiration  {
                    let mut errors = ValidationErrors::new();
                    let mut error = ValidationError::new("token");
                    error.add_param("message".into(), &"device token expired".to_string());
                    error.add_param("details".into(), &"no details".to_string());
                    errors.add("device", error);
                    return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::InvalidInput(serde_json::to_string(&errors).unwrap_or_default()) => token_duration, token_expiration));
                }

                let new_device = NewDevice::new(device_id, device_os, user_id, public_key);
                devices_repo.create(new_device.clone()).map_err(ectx!(try convert => new_device))?;
            }

            Ok(())
        }))
    }

    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        Box::new(self.storiqa_client.confirm_email(token.clone()).map_err(ectx!(convert => token)))
    }

    fn me(&self, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(cli.me(token.clone()).map_err(ectx!(convert => token)))
    }
    fn reset_password(&self, reset: ResetPassword) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(self.storiqa_client.reset_password(reset.clone()).map_err(ectx!(convert => reset)))
    }
    fn resend_email_verify(&self, resend: ResendEmailVerify) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(
            self.storiqa_client
                .resend_email_verify(resend.clone())
                .map_err(ectx!(convert => resend)),
        )
    }
    fn change_password(&self, change_password: ChangePassword, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(
            cli.change_password(change_password.clone(), token.clone())
                .map_err(ectx!(convert => change_password, token)),
        )
    }
    fn refresh_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(cli.refresh_jwt(token.clone()).map_err(ectx!(convert => token)))
    }
    fn revoke_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(cli.revoke_jwt(token.clone()).map_err(ectx!(convert => token)))
    }
    fn confirm_reset_password(&self, confirm: ResetPasswordConfirm) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let cli = self.storiqa_client.clone();
        Box::new(
            confirm
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_string(&e).unwrap_or_default()) => confirm))
                .into_future()
                .and_then(move |_| cli.confirm_reset_password(confirm.clone()).map_err(ectx!(convert => confirm))),
        )
    }
}
