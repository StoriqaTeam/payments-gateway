mod error;
mod responses;

pub use self::error::*;

use std::sync::Arc;

use base64;
use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use serde::Deserialize;
use serde_json;

use self::responses::*;
use super::HttpClient;
use config::Config;
use models::StoriqaJWT;
use models::*;
use utils::read_body;

pub trait StoriqaClient: Send + Sync + 'static {
    fn get_jwt(&self, email: String, password: Password) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn refresh_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn revoke_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn create_user(&self, new_user: NewUser) -> Box<Future<Item = User, Error = Error> + Send>;
    fn update_user(&self, update_user: UpdateUser, user_id: UserId, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send>;
    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn reset_password(&self, reset: ResetPassword) -> Box<Future<Item = (), Error = Error> + Send>;
    fn resend_email_verify(&self, resend: ResendEmailVerify) -> Box<Future<Item = (), Error = Error> + Send>;
    fn confirm_reset_password(&self, reset: ResetPasswordConfirm) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn change_password(&self, change: ChangePassword, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send>;
    fn me(&self, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send>;
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

    fn exec_query<T: for<'de> Deserialize<'de> + Send>(
        &self,
        query: &str,
        token: Option<StoriqaJWT>,
    ) -> impl Future<Item = GraphQLResponse<T>, Error = Error> + Send {
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
        let mut builder = Request::builder();
        builder.uri(self.storiqa_url.clone()).method(Method::POST);
        if let Some(token) = token {
            builder.header("Authorization", format!("Bearer {}", token.inner()));
        }
        builder
            .body(Body::from(body))
            .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput => query3))
            .into_future()
            .and_then(move |req| cli.request(req).map_err(ectx!(ErrorKind::Internal => query1)))
            .and_then(move |resp| read_body(resp.into_body()).map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorSource::Utf8, ErrorKind::Internal => bytes_clone))
            }).and_then(|string| {
                serde_json::from_str::<GraphQLResponse<T>>(&string).map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => string))
            })
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
            self.exec_query::<GetJWTByEmail>(&query, None)
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp_data| resp_data.get_jwt_by_email.token),
        )
    }

    fn get_jwt_by_oauth(&self, oauth_token: OauthToken, oauth_provider: Provider) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    getJWTByProvider(input: {{token: \"{}\", provider: {}, clientMutationId:\"\"}}) {{
                        token
                    }}
                }}
            "#,
            oauth_token,
            format!("{}", oauth_provider).to_uppercase(),
        );
        info!("{}", query);
        Box::new(
            self.exec_query::<GetJWTByProvider>(&query, None)
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp_data| resp_data.get_jwt_by_provider.token),
        )
    }

    fn create_user(&self, new_user: NewUser) -> Box<Future<Item = User, Error = Error> + Send> {
        let NewUser {
            email,
            password,
            first_name,
            last_name,
            device_type,
            ..
        } = new_user;
        let query = format!(
            r#"
                mutation M {{
                    createUser(input: {{email: \"{}\", password: \"{}\", firstName: \"{}\", lastName: \"{}\", device: {}, project: WALLET, clientMutationId:\"\"}}) {{
                        rawId
                        email
                        firstName
                        lastName
                    }}
                }}
            "#,
            email,
            password.inner(),
            first_name,
            last_name,
            device_type,
        );
        Box::new(
            self.exec_query::<CreateUser>(&query, None)
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp_data| resp_data.create_user),
        )
    }

    fn update_user(&self, update_user: UpdateUser, user_id: UserId, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send> {
        let UpdateUser {
            first_name,
            last_name,
            phone,
        } = update_user;
        let first_name = if let Some(text) = first_name {
            format!(r#",firstName: \"{}\""#, text)
        } else {
            "".to_string()
        };
        let last_name = if let Some(text) = last_name {
            format!(r#",lastName: \"{}\""#, text)
        } else {
            "".to_string()
        };
        let phone = if let Some(text) = phone {
            format!(r#",phone: \"{}\""#, text)
        } else {
            "".to_string()
        };
        let graphql_id = base64::encode(&format!("users|user|{}", user_id));
        let id = format!(r#"id: \"{}\""#, graphql_id);
        let query = format!(
            r#"
                mutation M {{
                    updateUser(input: {{{}{}{}{}, clientMutationId:\"\"}}) {{
                        rawId
                        email
                        firstName
                        lastName
                        phone
                    }}
                }}
            "#,
            id, first_name, last_name, phone,
        );
        Box::new(
            self.exec_query::<UpdateUserResponse>(&query, Some(token))
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp_data| resp_data.update_user),
        )
    }

    fn me(&self, token: StoriqaJWT) -> Box<Future<Item = User, Error = Error> + Send> {
        let query = r#"
                query M {
                    me {
                        rawId
                        email
                        firstName
                        lastName
                        phone
                    }
                }
            "#;
        Box::new(
            self.exec_query::<Me>(&query, Some(token))
                .and_then(|resp| {
                    resp.data
                        .clone()
                        .ok_or(ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp))
                }).map(|resp_data| resp_data.me),
        )
    }

    fn confirm_email(&self, token: EmailConfirmToken) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    verifyEmail(input: {{token: \"{}\", project: WALLET, clientMutationId:\"\"}}) {{
                        token
                    }}
                }}
            "#,
            token,
        );
        Box::new(
            self.exec_query::<GetEmailVerify>(&query, None)
                .and_then(|resp| {
                    resp.data
                        .clone()
                        .ok_or(ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp))
                }).map(|resp_data| resp_data.verify_email.token),
        )
    }
    fn reset_password(&self, reset: ResetPassword) -> Box<Future<Item = (), Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    requestPasswordReset(input: {{email: \"{}\", device: {}, project: WALLET, clientMutationId:\"\"}}) {{
                        success
                    }}
                }}
            "#,
            reset.email, reset.device,
        );
        Box::new(
            self.exec_query::<GetResetPassword>(&query, None)
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|_| ()),
        )
    }
    fn resend_email_verify(&self, resend: ResendEmailVerify) -> Box<Future<Item = (), Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    resendEmailVerificationLink(input: {{email: \"{}\", device: {}, project: WALLET, clientMutationId:\"\"}}) {{
                        success
                    }}
                }}
            "#,
            resend.email, resend.device,
        );
        Box::new(
            self.exec_query::<GetResendEmailVerify>(&query, None)
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|_| ()),
        )
    }
    fn confirm_reset_password(&self, reset: ResetPasswordConfirm) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    applyPasswordReset(input: {{token: \"{}\", password: \"{}\", project: WALLET, clientMutationId:\"\"}}) {{
                        success
                        token
                    }}
                }}
            "#,
            reset.token,
            reset.password.inner(),
        );
        Box::new(
            self.exec_query::<GetResetPasswordApply>(&query, None)
                .and_then(|resp| {
                    resp.data
                        .clone()
                        .ok_or(ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp))
                }).map(|resp_data| resp_data.apply_password_reset.token),
        )
    }
    fn change_password(&self, change: ChangePassword, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    changePassword(input: {{newPassword: \"{}\", oldPassword: \"{}\", clientMutationId:\"\"}}) {{
                        success
                    }}
                }}
            "#,
            change.new_password.inner(),
            change.old_password.inner(),
        );
        Box::new(
            self.exec_query::<GetChangePassword>(&query, Some(token))
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp| resp.request_password_change.token),
        )
    }
    fn refresh_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    refreshJWT
                }}
            "#
        );
        Box::new(
            self.exec_query::<GetRefreshJWT>(&query, Some(token))
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp| resp.refresh_jwt),
        )
    }
    fn revoke_jwt(&self, token: StoriqaJWT) -> Box<Future<Item = StoriqaJWT, Error = Error> + Send> {
        let query = format!(
            r#"
                mutation M {{
                    revokeJWT
                }}
            "#
        );
        Box::new(
            self.exec_query::<GetRevokeJWT>(&query, Some(token))
                .and_then(|resp| {
                    resp.data.clone().ok_or_else(|| {
                        if let Some(payload) = get_error_payload(resp.clone().errors) {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Validation(payload) => resp.clone())
                        } else {
                            ectx!(err ErrorContext::NoGraphQLData, ErrorKind::Unauthorized => resp.clone())
                        }
                    })
                }).map(|resp| resp.revoke_jwt),
        )
    }
}
