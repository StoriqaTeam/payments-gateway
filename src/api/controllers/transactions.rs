use failure::Fail;
use futures::prelude::*;

use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use api::error::*;
use api::requests::*;
use api::responses::*;
use models::*;
use serde_qs;

pub fn post_transactions(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let maybe_token = ctx.get_auth_token();
    let body = ctx.body.clone();
    Box::new(
        maybe_token
            .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
            .into_future()
            .and_then(move |token| {
                parse_body::<PostTransactionsRequest>(body)
                    .and_then(move |input| input.to_create().into_future())
                    .and_then(move |create| {
                        let create_clone = create.clone();
                        transactions_service
                            .create_transaction(token, create)
                            .map_err(ectx!(convert => create_clone))
                            .and_then(|transactions| {
                                let transactions: Vec<TransactionsResponse> = transactions.into_iter().map(From::from).collect();
                                response_with_model(&transactions)
                            })
                    })
            }),
    )
}

pub fn post_rate(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let maybe_token = ctx.get_auth_token();
    let body = ctx.body.clone();
    Box::new(
        maybe_token
            .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
            .into_future()
            .and_then(move |_token| {
                parse_body::<PostRateRequest>(body).and_then(move |rate| {
                    let rate_clone = rate.clone();
                    transactions_service
                        .get_rate(rate.into())
                        .map_err(ectx!(convert => rate_clone))
                        .and_then(|rate| response_with_model(&RateResponse::from(rate)))
                })
            }),
    )
}

pub fn get_users_transactions(ctx: &Context, user_id: UserId) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let maybe_token = ctx.get_auth_token();
    let path_and_query = ctx.uri.path_and_query();
    let path_and_query_clone = ctx.uri.path_and_query();
    Box::new(
        ctx.uri
            .query()
            .ok_or(ectx!(err ErrorContext::RequestMissingQuery, ErrorKind::BadRequest => path_and_query))
            .and_then(|query| {
                serde_qs::from_str::<GetUsersTransactionsParams>(query).map_err(|e| {
                    let e = format_err!("{}", e);
                    ectx!(err e, ErrorContext::RequestQueryParams, ErrorKind::BadRequest => path_and_query_clone)
                })
            }).into_future()
            .and_then(move |input| {
                maybe_token
                    .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
                    .into_future()
                    .and_then(move |token| {
                        let input_clone = input.clone();
                        transactions_service
                            .get_transactions_for_user(token, user_id, input.offset, input.limit)
                            .map_err(ectx!(convert => input_clone))
                    })
            }).and_then(|transactions| {
                let transactions: Vec<TransactionsResponse> = transactions.into_iter().map(From::from).collect();
                response_with_model(&transactions)
            }),
    )
}

pub fn get_accounts_transactions(ctx: &Context, account_id: AccountId) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let maybe_token = ctx.get_auth_token();
    let path_and_query = ctx.uri.path_and_query();
    let path_and_query_clone = ctx.uri.path_and_query();
    Box::new(
        ctx.uri
            .query()
            .ok_or(ectx!(err ErrorContext::RequestMissingQuery, ErrorKind::BadRequest => path_and_query))
            .and_then(|query| {
                serde_qs::from_str::<GetUsersTransactionsParams>(query).map_err(|e| {
                    let e = format_err!("{}", e);
                    ectx!(err e, ErrorContext::RequestQueryParams, ErrorKind::BadRequest => path_and_query_clone)
                })
            }).into_future()
            .and_then(move |input| {
                maybe_token
                    .ok_or_else(|| ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
                    .into_future()
                    .and_then(move |token| {
                        transactions_service
                            .get_account_transactions(token, account_id, input.offset, input.limit)
                            .map_err(ectx!(convert))
                    })
            }).and_then(|transactions| {
                let transactions: Vec<TransactionsResponse> = transactions.into_iter().map(From::from).collect();
                response_with_model(&transactions)
            }),
    )
}
