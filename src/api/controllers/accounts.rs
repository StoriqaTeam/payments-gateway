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

pub fn post_accounts(ctx: &Context, user_id: UserId) -> ControllerFuture {
    let accounts_service = ctx.accounts_service.clone();
    let body = ctx.body.clone();
    Box::new(
        ctx.authenticate()
            .and_then(move |user_id_auth| {
                if user_id == user_id_auth {
                    future::ok(())
                } else {
                    future::err(ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
                }
            }).and_then(move |_| {
                parse_body::<PostAccountsRequest>(body)
                    .and_then(move |input| {
                        let input_clone = input.clone();
                        accounts_service
                            .create_account((input, user_id).into())
                            .map_err(ectx!(convert => input_clone))
                    }).and_then(|account| response_with_model(&AccountsResponse::from(account)))
            }),
    )
}

pub fn get_users_accounts(ctx: &Context, user_id: UserId) -> ControllerFuture {
    let accounts_service = ctx.accounts_service.clone();
    let uri = ctx.uri.clone();
    Box::new(
        ctx.authenticate()
            .and_then(move |user_id_auth| {
                if user_id == user_id_auth {
                    future::ok(())
                } else {
                    future::err(ectx!(err ErrorContext::Token, ErrorKind::Unauthorized))
                }
            }).and_then(move |_| {
                let uri_clone = uri.clone();
                uri.clone()
                    .query()
                    .ok_or(ectx!(err ErrorContext::RequestMissingQuery, ErrorKind::BadRequest => uri))
                    .and_then(|query| {
                        serde_qs::from_str::<GetUsersAccountsParams>(query).map_err(|e| {
                            let e = format_err!("{}", e);
                            ectx!(err e, ErrorContext::RequestQueryParams, ErrorKind::BadRequest => uri_clone)
                        })
                    }).into_future()
            }).and_then(move |input| {
                let input_clone = input.clone();
                accounts_service
                    .get_accounts_for_user(user_id, input.offset, input.limit)
                    .map_err(ectx!(convert => input_clone))
            }).and_then(|accounts| {
                let accounts: Vec<AccountsResponse> = accounts.into_iter().map(From::from).collect();
                response_with_model(&accounts)
            }),
    )
}

pub fn get_accounts(ctx: &Context, account_id: AccountId) -> ControllerFuture {
    let accounts_service = ctx.accounts_service.clone();
    Box::new(ctx.authenticate().and_then(move |user_id_auth| {
        accounts_service
            .get_account(user_id_auth, account_id)
            .map_err(ectx!(convert))
            .and_then(|account| response_with_model(&account.map(AccountsResponse::from)))
    }))
}

pub fn put_accounts(ctx: &Context, account_id: AccountId) -> ControllerFuture {
    let accounts_service = ctx.accounts_service.clone();
    let body = ctx.body.clone();
    Box::new(ctx.authenticate().and_then(move |user_id_auth| {
        parse_body::<PutAccountsRequest>(body)
            .and_then(move |input| {
                let input_clone = input.clone();
                accounts_service
                    .update_account(user_id_auth, account_id, input.into())
                    .map_err(ectx!(convert => input_clone))
            }).and_then(|account| response_with_model(&AccountsResponse::from(account)))
    }))
}

pub fn delete_accounts(ctx: &Context, account_id: AccountId) -> ControllerFuture {
    let accounts_service = ctx.accounts_service.clone();
    Box::new(ctx.authenticate().and_then(move |user_id_auth| {
        accounts_service
            .delete_account(user_id_auth, account_id)
            .map_err(ectx!(convert))
            .and_then(|account| response_with_model(&AccountsResponse::from(account)))
    }))
}
