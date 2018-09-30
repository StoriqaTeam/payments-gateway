use super::super::error::*;
use super::super::requests::*;
use super::super::responses::*;
use super::response_with_model;
use super::Context;
use super::ControllerFuture;
use failure::Fail;
use futures::prelude::*;
use serde_json;

pub fn post_sessions(ctx: &Context) -> ControllerFuture {
    let cli = ctx.storiqa_client.clone();
    Box::new(
        String::from_utf8(ctx.body.clone())
            .map_err(ewrap!(ErrorSource::RequestUTF8, ErrorKind::BadRequest => ctx.body))
            .into_future()
            .and_then(|string| {
                serde_json::from_str::<PostSessionsRequest>(&string).map_err(ewrap!(
                    ErrorSource::RequestJson,
                    ErrorKind::BadRequest
                    => string
                ))
            }).and_then(move |input| {
                let input_clone = input.clone();
                cli.get_jwt(input.email, input.password).map_err(ewrap!(catch => input_clone))
            }).and_then(|jwt| {
                let model = PostSessionsResponse { token: jwt };
                response_with_model(&model)
            }),
    )
}
