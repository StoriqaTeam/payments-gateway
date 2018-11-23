use super::super::requests::*;
use super::super::responses::*;
use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use api::error::*;
use failure::Fail;
use futures::prelude::*;

pub fn post_sessions(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let auth_service = ctx.auth_service.clone();
    let body = ctx.body.clone();
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
                    }).and_then(move |jwt| {
                        let token = jwt.clone();
                        let auth_service_clone = auth_service.clone();
                        auth_service
                            .get_jwt_auth(jwt.clone())
                            .map_err(ectx!(convert => token))
                            .into_future()
                            .and_then(move |auth| {
                                auth_service_clone
                                    .authenticate(auth_info.clone(), auth.user_id)
                                    .map_err(ectx!(convert => auth_info, auth.user_id))
                            }).map(|_| jwt)
                    }).and_then(|jwt| {
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
                    }).and_then(move |jwt| {
                        let token = jwt.clone();
                        let auth_service_clone = auth_service.clone();
                        auth_service
                            .get_jwt_auth(jwt.clone())
                            .map_err(ectx!(convert => token))
                            .into_future()
                            .and_then(move |auth| {
                                auth_service_clone
                                    .authenticate(auth_info.clone(), auth.user_id)
                                    .map_err(ectx!(convert => auth_info, auth.user_id))
                            }).map(|_| jwt)
                    }).and_then(|jwt| {
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
            }).and_then(move |user| {
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
            }).and_then(move |user| response_with_model(&user)),
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
            }).and_then(|token| response_with_model(&token)),
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
            }).and_then(|token| response_with_model(&token)),
    )
}

pub fn post_users_confirm_add_device(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    let body = ctx.body.clone();

    Box::new(
        parse_body::<PostUsersConfirmAddDeviceRequest>(body)
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service.confirm_add_device(input.token).map_err(ectx!(convert => input_clone))
            }).and_then(|token| response_with_model(&token)),
    )
}

pub fn post_users_reset_password(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    Box::new(
        parse_body::<PostUsersResetPasswordRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service.reset_password(input.into()).map_err(ectx!(convert => input_clone))
            }).and_then(|token| response_with_model(&token)),
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
            }).and_then(|token| response_with_model(&token)),
    )
}

pub fn post_users_change_password(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
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
                    }).and_then(|token| response_with_model(&token))
            })
    }))
}

pub fn post_users_confirm_reset_password(ctx: &Context) -> ControllerFuture {
    let users_service = ctx.users_service.clone();
    Box::new(
        parse_body::<PostUsersConfirmResetPasswordRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                users_service
                    .confirm_reset_password(input.into())
                    .map_err(ectx!(convert => input_clone))
            }).and_then(|token| response_with_model(&token)),
    )
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
