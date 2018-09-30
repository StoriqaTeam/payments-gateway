use super::super::requests::*;
use super::super::responses::*;
use super::super::utils::{parse_body, response_with_model};
use super::authorize;
use super::Context;
use super::ControllerFuture;
use failure::Fail;
use futures::prelude::*;

pub fn post_sessions(ctx: &Context) -> ControllerFuture {
    let cli = ctx.storiqa_client.clone();
    Box::new(
        parse_body::<PostSessionsRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                cli.get_jwt(input.email, input.password).map_err(ewrap!(catch => input_clone))
            }).and_then(|jwt| {
                let model = PostSessionsResponse { token: jwt };
                response_with_model(&model)
            }),
    )
}

pub fn post_users(ctx: &Context) -> ControllerFuture {
    let cli = ctx.storiqa_client.clone();
    Box::new(
        parse_body::<PostUsersRequest>(ctx.body.clone())
            .and_then(move |input| {
                let input_clone = input.clone();
                cli.create_user(input.email, input.password, input.first_name, input.last_name)
                    .map_err(ewrap!(catch => input_clone))
            }).and_then(|user| response_with_model(&user)),
    )
}

pub fn get_users_me(ctx: &Context) -> ControllerFuture {
    let cli = ctx.storiqa_client.clone();
    Box::new(
        authorize(ctx)
            .and_then(move |auth| cli.me(auth.token).map_err(ewrap!(catch)))
            .and_then(|user| response_with_model(&user)),
    )
}
