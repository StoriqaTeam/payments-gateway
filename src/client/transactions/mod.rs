mod error;
mod requests;
mod responses;

pub use self::error::*;

use std::sync::Arc;

use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use serde::Deserialize;
use serde_json;

use self::responses::*;
use super::HttpClient;
use config::Config;
use models::*;
use utils::read_body;

pub trait TransactionsClient: Send + Sync + 'static {
    fn create_account(&self, input: CreateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send>;
    fn update_account(&self, account_id: AccountId, payload: UpdateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send>;
    fn delete_account(&self, account_id: AccountId) -> Box<Future<Item = AccountResponse, Error = Error> + Send>;
    fn get_account_balance(&self, account_id: AccountId) -> Box<Future<Item = BalanceResponse, Error = Error> + Send>;
    fn create_transaction(&self, input: CreateTransaction) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send>;
    fn get_account_transactions(&self, account_id: AccountId) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send>;
}

pub struct TransactionsClientImpl {
    cli: Arc<HttpClient>,
    transactions_url: String,
    token: AuthenticationToken,
    workspace_id: WorkspaceId,
}

impl TransactionsClientImpl {
    pub fn new<C: HttpClient>(config: &Config, cli: C) -> Self {
        Self {
            cli: Arc::new(cli),
            transactions_url: config.client.transactions_url.clone(),
            token: config.auth.storiqa_transactions_token.clone(),
            workspace_id: config.auth.storiqa_transactions_user_id.clone(),
        }
    }

    fn exec_query<T: for<'de> Deserialize<'de> + Send>(
        &self,
        query: &str,
        body: String,
        method: Method,
    ) -> impl Future<Item = T, Error = Error> + Send {
        let query = query.to_string();
        let query1 = query.clone();
        let query2 = query.clone();
        let query3 = query.clone();
        let cli = self.cli.clone();
        let mut builder = Request::builder();
        let url = format!("{}{}", self.transactions_url, query);
        builder.uri(url).method(method);
        builder.header("Authorization", format!("Bearer {}", self.token.raw()));
        builder
            .body(Body::from(body))
            .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput => query3))
            .into_future()
            .and_then(move |req| cli.request(req).map_err(ectx!(ErrorKind::Internal => query1)))
            .and_then(move |resp| read_body(resp.into_body()).map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorSource::Utf8, ErrorKind::Internal => bytes_clone))
            }).and_then(|string| serde_json::from_str::<T>(&string).map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => string)))
    }
}

impl TransactionsClient for TransactionsClientImpl {
    fn create_account(&self, input: CreateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        unimplemented!()
    }
    fn update_account(&self, account_id: AccountId, payload: UpdateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        unimplemented!()
    }
    fn delete_account(&self, account_id: AccountId) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        unimplemented!()
    }
    fn get_account_balance(&self, account_id: AccountId) -> Box<Future<Item = BalanceResponse, Error = Error> + Send> {
        unimplemented!()
    }
    fn create_transaction(&self, input: CreateTransaction) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send> {
        unimplemented!()
    }
    fn get_account_transactions(&self, account_id: AccountId) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send> {
        unimplemented!()
    }
}
