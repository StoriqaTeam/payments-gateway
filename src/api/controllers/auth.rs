use super::super::error::*;
use super::super::requests::*;
use super::super::responses::*;
use super::response_with_model;
use super::Context;
use super::ControllerFuture;
use client::ErrorKind as ClientErrorKind;
use failure::Fail;
use futures::prelude::*;
use hyper::{Body, Response};
use serde_json;

pub fn post_sessions(ctx: &Context) -> ControllerFuture {
    let cli = ctx.storiqa_client.clone();
    Box::new(
        String::from_utf8(ctx.body.clone())
            .map_err(|e| error_context!(e, ErrorContext::RequestUTF8, ErrorKind::UnprocessableEntity, ctx.body))
            .into_future()
            .and_then(|string| {
                serde_json::from_str::<PostSessionsRequest>(&string)
                    .map_err(move |e| error_context!(e, ErrorContext::RequestJson, ErrorKind::UnprocessableEntity, string))
            })
            .and_then(move |input| {
                let input_clone = input.clone();
                cli.getJWT(input.email, input.password).map_err(move |e| {
                    let kind = e.kind().into();
                    error_context!(e, ErrorContext::Client, kind, input_clone)
                })
            })
            .and_then(|jwt| {
                let model = PostSessionsResponse { token: jwt };
                response_with_model(&model)
            }),
    )
}
