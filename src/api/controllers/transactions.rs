use failure::Fail;
use futures::future;
use futures::prelude::*;

use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use api::error::*;
use api::requests::*;
use api::responses::*;
use models::*;
use serde_qs;

pub fn get_transaction(ctx: &Context, tx_id: TransactionId) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    Box::new(ctx.authenticate().and_then(move |user_id_auth| {
        transactions_service
            .get_transaction(user_id_auth, tx_id)
            .map_err(ectx!(convert => user_id_auth, tx_id))
            .and_then(|tx| response_with_model(&tx.map(TransactionsResponse::from)))
    }))
}

pub fn post_transactions(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let body = ctx.body.clone();
    Box::new(ctx.authenticate().and_then(move |user_id_auth| {
        parse_body::<PostTransactionsRequest>(body)
            .and_then(move |input| input.to_create().into_future())
            .and_then(move |create| {
                let create_clone = create.clone();
                transactions_service
                    .create_transaction(user_id_auth, create)
                    .map_err(ectx!(convert => create_clone))
                    .and_then(|transaction| response_with_model(&TransactionsResponse::from(transaction)))
            })
    }))
}

pub fn post_rate(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let body = ctx.body.clone();
    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        parse_body::<PostRateRequest>(body).and_then(move |rate| {
            let rate_clone = rate.clone();
            transactions_service
                .get_rate(rate.into())
                .map_err(ectx!(convert => rate_clone))
                .and_then(|rate| response_with_model(&RateResponse::from(rate)))
        })
    }))
}

pub fn post_rate_refresh(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let body = ctx.body.clone();
    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        parse_body::<PostRateRefreshRequest>(body).and_then(move |rate| {
            let rate_clone = rate.clone();
            transactions_service
                .refresh_rate(rate.into())
                .map_err(ectx!(convert => rate_clone))
                .and_then(|rate| response_with_model(&RateRefreshResponse::from(rate)))
        })
    }))
}

pub fn post_fees(ctx: &Context) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let body = ctx.body.clone();
    Box::new(ctx.authenticate().and_then(move |_user_id_auth| {
        parse_body::<PostFeesRequest>(body).and_then(move |fees| {
            let fees_clone = fees.clone();
            transactions_service
                .get_fees(fees.into())
                .map_err(ectx!(convert => fees_clone))
                .and_then(|fees| response_with_model(&FeesResponse::from(fees)))
        })
    }))
}

pub fn get_users_transactions(ctx: &Context, user_id: UserId) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let uri = ctx.uri.clone();
    Box::new(
        ctx.authenticate()
            .and_then(move |user_id_auth| {
                if user_id == user_id_auth {
                    future::ok(())
                } else {
                    future::err(ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
                }
            })
            .and_then(move |_| {
                let uri_clone = uri.clone();
                uri.clone()
                    .query()
                    .ok_or(ectx!(err ErrorContext::RequestMissingQuery, ErrorKind::BadRequest => uri))
                    .and_then(|query| {
                        serde_qs::from_str::<GetUsersTransactionsParams>(query).map_err(|e| {
                            let e = format_err!("{}", e);
                            ectx!(err e, ErrorContext::RequestQueryParams, ErrorKind::BadRequest => uri_clone)
                        })
                    })
                    .into_future()
            })
            .and_then(move |input| {
                let input_clone = input.clone();
                transactions_service
                    .get_transactions_for_user(user_id, input.offset, input.limit)
                    .map_err(ectx!(convert => input_clone))
            })
            .and_then(|transactions| {
                let transactions: Vec<TransactionsResponse> = transactions.into_iter().map(From::from).collect();
                response_with_model(&transactions)
            }),
    )
}

pub fn get_accounts_transactions(ctx: &Context, account_id: AccountId) -> ControllerFuture {
    let transactions_service = ctx.transactions_service.clone();
    let uri = ctx.uri.clone();
    Box::new(
        ctx.authenticate()
            .and_then(move |user_id_auth| {
                let uri_clone = uri.clone();
                let uri_clone2 = uri.clone();
                let uri_query = uri.query();
                uri_query
                    .ok_or(ectx!(err ErrorContext::RequestMissingQuery, ErrorKind::BadRequest => uri_clone))
                    .and_then(move |query| {
                        serde_qs::from_str::<GetUsersTransactionsParams>(query).map_err(|e| {
                            let e = format_err!("{}", e);
                            ectx!(err e, ErrorContext::RequestQueryParams, ErrorKind::BadRequest => uri_clone2)
                        })
                    })
                    .map(|input| (user_id_auth, input))
                    .into_future()
            })
            .and_then(move |(user_id_auth, input)| {
                transactions_service
                    .get_account_transactions(user_id_auth, account_id, input.offset, input.limit)
                    .map_err(ectx!(convert))
            })
            .and_then(|transactions| {
                let transactions: Vec<TransactionsResponse> = transactions.into_iter().map(From::from).collect();
                response_with_model(&transactions)
            }),
    )
}
