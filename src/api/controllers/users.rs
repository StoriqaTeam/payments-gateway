use chrono::NaiveDateTime;
use failure::Fail;
use futures::future::{self, Either};
use futures::prelude::*;

use super::super::requests::*;
use super::super::responses::*;
use super::super::utils::{parse_body, response_with_model, response_with_redirect};
use super::Context;
use super::ControllerFuture;
use api::error::*;
use models::*;

pub fn merge_user(ctx: Context, jwt: StoriqaJWT) -> Box<Future<Item = (), Error = Error> + Send> {
    let users_service = ctx.users_service.clone();
    let accounts_service = ctx.accounts_service.clone();
    let users_service_clone = users_service.clone();
    Box::new(users_service.me(jwt.clone()).map_err(ectx!(convert => jwt)).and_then(move |user| {
        let user_id = user.id.clone();
        users_service_clone
            .merge_user(user.clone())
            .map_err(ectx!(convert => user))
            .and_then(move |new_user| {
                if new_user {
                    Either::A(
                        accounts_service
                            .create_default_accounts(user_id)
                            .map_err(ectx!(convert => user_id))
                            .map(|_| ()),
                    )
                } else {
                    Either::B(future::ok(()))
                }
            })
    }))
}

pub fn post_sessions(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let auth_service = ctx.auth_service.clone();
    let body = ctx.body.clone();
    let ctx_clone = ctx.clone();
    Box::new(
        ctx.get_auth_info()
            .into_future()
            .map_err(ectx!(convert))
            .and_then(move |auth_info| {
                parse_body::<PostSessionsRequest>(body)
                    .and_then(move |input| {
                        let input_clone = input.clone();
                        users_service
                            .get_jwt(input.email, input.password)
                            .map_err(ectx!(convert => input_clone))
                            .and_then(move |jwt| merge_user(ctx_clone, jwt.clone()).map(|_| jwt))
                    })
                    .and_then(move |jwt| {
                        let token = jwt.clone();
                        let token_clone = jwt.clone();
                        let auth_service_clone = auth_service.clone();
                        let auth_service_clone2 = auth_service.clone();
                        auth_service
                            .get_jwt_auth(jwt.clone())
                            .map_err(ectx!(convert => token))
                            .into_future()
                            .and_then(move |auth| {
                                auth_service_clone
                                    .get_exp(token_clone.clone())
                                    .map_err(ectx!(convert => token_clone))
                                    .map(|exp| (auth, exp))
                            })
                            .and_then(move |(auth, exp)| {
                                auth_service_clone2
                                    .authenticate(auth_info.clone(), auth.user_id, exp)
                                    .map_err(ectx!(convert => auth_info, auth.user_id, exp))
                            })
                            .map(|_| jwt)
                    })
                    .and_then(|jwt| {
                        let model = PostSessionsResponse { token: jwt };
                        response_with_model(&model)
                    })
            }),
    )
}

pub fn post_sessions_oauth(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let auth_service = ctx.auth_service.clone();
    let body = ctx.body.clone();
    let ctx_clone = ctx.clone();
    Box::new(
        ctx.get_auth_info()
            .into_future()
            .map_err(ectx!(convert))
            .and_then(move |auth_info| {
                parse_body::<PostSessionsOauthRequest>(body)
                    .and_then(move |input| {
                        let input_clone = input.clone();
                        users_service
                            .get_jwt_by_oauth(input.oauth_token, input.oauth_provider)
                            .map_err(ectx!(convert => input_clone))
                            .and_then(move |jwt| merge_user(ctx_clone, jwt.clone()).map(|_| jwt))
                    })
                    .and_then(move |jwt| {
                        let token = jwt.clone();
                        let token_clone = jwt.clone();
                        let auth_service_clone = auth_service.clone();
                        let auth_service_clone2 = auth_service.clone();
                        auth_service
                            .get_jwt_auth(jwt.clone())
                            .map_err(ectx!(convert => token))
                            .into_future()
                            .and_then(move |auth| {
                                auth_service_clone
                                    .get_exp(token_clone.clone())
                                    .map_err(ectx!(convert => token_clone))
                                    .map(|exp| (auth, exp))
                            })
                            .and_then(move |(auth, exp)| {
                                auth_service_clone2
                                    .authenticate(auth_info.clone(), auth.user_id, exp)
                                    .map_err(ectx!(convert => auth_info, auth.user_id, exp))
                            })
                            .map(|_| jwt)
                    })
                    .and_then(|jwt| {
                        let model = PostSessionsResponse { token: jwt };
                        response_with_model(&model)
                    })
            }),
    )
}

pub fn post_users(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let accounts_service = ctx.accounts_service.clone();
    Box::new(
        parse_body::<PostUsersRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service.create_user(input.into()).map_err(ectx!(convert => input_clone))
            })
            .and_then(move |user| {
                accounts_service
                    .create_default_accounts(user.id)
                    .then(move |_| response_with_model(&user))
            }),
    )
}

pub fn put_users(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let body = ctx.body.clone();
    let maybe_token = ctx.get_auth_token();
    Box::new(
        ctx.authenticate()
            .and_then(move |user_id_auth| {
                maybe_token
                    .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
                    .into_future()
                    .and_then(move |token| {
                        parse_body::<PutUsersRequest>(body).and_then(move |input| {
                            let input_clone = input.clone();
                            users_service
                                .update_user(input.into(), user_id_auth, token)
                                .map_err(ectx!(convert => input_clone))
                        })
                    })
            })
            .and_then(move |user| response_with_model(&user)),
    )
}

pub fn post_users_confirm_email(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    Box::new(
        parse_body::<PostUsersConfirmEmailRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service
                    .confirm_email(input.email_confirm_token)
                    .map_err(ectx!(convert => input_clone))
            })
            .and_then(|token| response_with_model(&token)),
    )
}

pub fn get_users_confirm_email(ctx: &Context, token: EmailConfirmToken) -> ControllerFuture {
    let confirm_email_url = ctx.config.redirections.confirm_email_url.clone();
    Box::new(
        ctx.users_service
            .confirm_email(token.clone())
            .map_err(ectx!(convert => token))
            .and_then(|_| response_with_redirect(confirm_email_url)),
    )
}

pub fn post_users_add_device(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let body = ctx.body.clone();
    Box::new(
        parse_body::<PostUsersAddDeviceRequest>(body)
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service
                    .add_device(input.device_id, input.device_os, input.public_key, input.user_id)
                    .map_err(ectx!(convert => input_clone))
            })
            .and_then(|token| response_with_model(&token)),
    )
}

pub fn post_users_confirm_add_device(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let body = ctx.body.clone();

    Box::new(
        parse_body::<PostUsersConfirmAddDeviceRequest>(body)
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service
                    .confirm_add_device(input.token, Some(input.device_id))
                    .map_err(ectx!(convert => input_clone))
            })
            .and_then(|token| response_with_model(&token)),
    )
}

pub fn get_register_device(ctx: &Context, token: DeviceConfirmToken) -> ControllerFuture {
    let confirm_register_device_url = ctx.config.redirections.confirm_register_device_url.clone();
    Box::new(
        ctx.users_service
            .confirm_add_device(token, None)
            .map_err(ectx!(convert => token))
            .and_then(|_| response_with_redirect(confirm_register_device_url)),
    )
}

pub fn post_users_reset_password(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    Box::new(
        parse_body::<PostUsersResetPasswordRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service.reset_password(input.into()).map_err(ectx!(convert => input_clone))
            })
            .and_then(|token| response_with_model(&token)),
    )
}

pub fn post_users_resend_confirm_email(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    Box::new(
        parse_body::<PostUsersResendEmailVerifyRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service
                    .resend_email_verify(input.into())
                    .map_err(ectx!(convert => input_clone))
            })
            .and_then(|token| response_with_model(&token)),
    )
}

pub fn post_users_change_password(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let users_service_clone = users_service.clone();
    let auth_service = ctx.auth_service.clone();
    let maybe_token = ctx.get_auth_token();
    let body = ctx.body.clone();

    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        maybe_token
            .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
            .into_future()
            .and_then(move |token| {
                parse_body::<PostUsersChangePasswordRequest>(body)
                    .and_then(move |input| {
                        let input_clone = input.clone();
                        users_service
                            .change_password(input.into(), token)
                            .map_err(ectx!(convert => input_clone))
                    })
                    .and_then(move |token| {
                        let token_clone = token.clone();
                        let token_clone2 = token.clone();
                        let auth_service_clone = auth_service.clone();
                        auth_service
                            .get_exp(token_clone.clone())
                            .map_err(ectx!(convert => token_clone))
                            .into_future()
                            .and_then(move |exp| {
                                auth_service_clone
                                    .get_jwt_auth(token_clone2.clone())
                                    .map_err(ectx!(convert => token_clone2))
                                    .into_future()
                                    .and_then(move |auth| {
                                        users_service_clone
                                            .revoke_tokens_db(auth.user_id, NaiveDateTime::from_timestamp(exp as i64, 0))
                                            .map_err(ectx!(convert => auth.user_id))
                                    })
                            })
                            .map(|_| token)
                    })
                    .and_then(|token| response_with_model(&token))
            })
    }))
}

pub fn post_users_confirm_reset_password(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let users_service_clone = users_service.clone();
    let auth_service = ctx.auth_service.clone();
    Box::new(
        parse_body::<PostUsersConfirmResetPasswordRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service
                    .confirm_reset_password(input.into())
                    .map_err(ectx!(convert => input_clone))
            })
            .and_then(move |token| {
                let token_clone = token.clone();
                let token_clone2 = token.clone();
                let auth_service_clone = auth_service.clone();
                auth_service
                    .get_exp(token_clone.clone())
                    .map_err(ectx!(convert => token_clone))
                    .into_future()
                    .and_then(move |exp| {
                        auth_service_clone
                            .get_jwt_auth(token_clone2.clone())
                            .map_err(ectx!(convert => token_clone2))
                            .into_future()
                            .and_then(move |auth| {
                                users_service_clone
                                    .revoke_tokens_db(auth.user_id, NaiveDateTime::from_timestamp(exp as i64, 0))
                                    .map_err(ectx!(convert => auth.user_id))
                            })
                    })
                    .map(|_| token)
            })
            .and_then(|token| response_with_model(&token)),
    )
}

pub fn get_confirm_reset_password(ctx: &Context, _token: PasswordResetToken) -> ControllerFuture {
    let confirm_reset_password_url = ctx.config.redirections.confirm_reset_password_url.clone();
    response_with_redirect(confirm_reset_password_url)
}

pub fn get_users_me(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let maybe_token = ctx.get_auth_token();

    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        maybe_token
            .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
            .into_future()
            .and_then(move |token| {
                users_service
                    .me(token)
                    .map_err(ectx!(convert))
                    .and_then(|user| response_with_model(&user))
            })
    }))
}

pub fn post_sessions_refresh(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let maybe_token = ctx.get_auth_token();

    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        maybe_token
            .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
            .into_future()
            .and_then(move |token| {
                users_service
                    .refresh_jwt(token)
                    .map_err(ectx!(convert))
                    .and_then(|user| response_with_model(&user))
            })
    }))
}

pub fn post_sessions_revoke(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let users_service_clone = users_service.clone();
    let auth_service = ctx.auth_service.clone();
    let maybe_token = ctx.get_auth_token();

    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        maybe_token
            .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
            .into_future()
            .and_then(move |token| {
                users_service
                    .revoke_jwt(token)
                    .map_err(ectx!(convert))
                    .and_then(move |token| {
                        let token_clone = token.clone();
                        let token_clone2 = token.clone();
                        let auth_service_clone = auth_service.clone();
                        auth_service
                            .get_exp(token_clone.clone())
                            .map_err(ectx!(convert => token_clone))
                            .into_future()
                            .and_then(move |exp| {
                                auth_service_clone
                                    .get_jwt_auth(token_clone2.clone())
                                    .map_err(ectx!(convert => token_clone2))
                                    .into_future()
                                    .and_then(move |auth| {
                                        users_service_clone
                                            .revoke_tokens_db(auth.user_id, NaiveDateTime::from_timestamp(exp as i64, 0))
                                            .map_err(ectx!(convert => auth.user_id))
                                    })
                            })
                            .map(|_| token)
                    })
                    .and_then(|token| response_with_model(&token))
            })
    }))
}
