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

use self::requests::*;
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
    fn create_transaction(&self, input: CreateTransaction) -> Box<Future<Item = TransactionResponse, Error = Error> + Send>;
    fn get_account_transactions(
        &self,
        account_id: AccountId,
        offset: i64,
        limit: i64,
    ) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send>;
    fn get_rate(&self, input: GetRate) -> Box<Future<Item = Rate, Error = Error> + Send>;
    fn get_fees(&self, input: GetFees) -> Box<Future<Item = Fees, Error = Error> + Send>;
}

#[derive(Clone)]
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
        body: Option<String>,
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
        let body = if let Some(body) = body { Body::from(body) } else { Body::empty() };
        builder
            .body(body)
            .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput => query3))
            .into_future()
            .and_then(move |req| cli.request(req).map_err(ectx!(convert => query1)))
            .and_then(move |resp| read_body(resp.into_body()).map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorSource::Utf8, ErrorKind::Internal => bytes_clone))
            })
            .and_then(|string| serde_json::from_str::<T>(&string).map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => string)))
    }
}

impl TransactionsClient for TransactionsClientImpl {
    fn create_account(&self, input: CreateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        let client = self.clone();
        let workspace_id = self.workspace_id;
        let create: CreateAccountRequest = (input, workspace_id).into();
        let url = format!("/accounts");
        Box::new(
            serde_json::to_string(&create)
                .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => create))
                .into_future()
                .and_then(move |body| client.exec_query::<AccountResponse>(&url, Some(body), Method::POST)),
        )
    }
    fn update_account(&self, account_id: AccountId, payload: UpdateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        let client = self.clone();
        let url = format!("/accounts/{}", account_id);
        Box::new(
            serde_json::to_string(&payload)
                .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => payload))
                .into_future()
                .and_then(move |body| client.exec_query::<AccountResponse>(&url, Some(body), Method::PUT)),
        )
    }
    fn delete_account(&self, account_id: AccountId) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        let client = self.clone();
        let url = format!("/accounts/{}", account_id);
        Box::new(client.exec_query::<AccountResponse>(&url, None, Method::DELETE))
    }
    fn get_account_balance(&self, account_id: AccountId) -> Box<Future<Item = BalanceResponse, Error = Error> + Send> {
        let client = self.clone();
        let url = format!("/accounts/{}/balances", account_id);
        Box::new(client.exec_query::<BalanceResponse>(&url, None, Method::GET))
    }
    fn create_transaction(&self, input: CreateTransaction) -> Box<Future<Item = TransactionResponse, Error = Error> + Send> {
        let client = self.clone();
        let workspace_id = self.workspace_id;
        let create: CreateTransactionRequest = (input, workspace_id).into();
        let url = format!("/transactions");
        Box::new(
            serde_json::to_string(&create)
                .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => create))
                .into_future()
                .and_then(move |body| client.exec_query::<TransactionResponse>(&url, Some(body), Method::POST)),
        )
    }
    fn get_account_transactions(
        &self,
        account_id: AccountId,
        offset: i64,
        limit: i64,
    ) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send> {
        let client = self.clone();
        let url = format!("/accounts/{}/transactions?offset={}&limit={}", account_id, offset, limit);
        Box::new(client.exec_query::<Vec<TransactionResponse>>(&url, None, Method::GET))
    }
    fn get_rate(&self, input: GetRate) -> Box<Future<Item = Rate, Error = Error> + Send> {
        let client = self.clone();
        let url = format!("/rate");
        Box::new(
            serde_json::to_string(&input)
                .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => input))
                .into_future()
                .and_then(move |body| client.exec_query::<Rate>(&url, Some(body), Method::POST)),
        )
    }
    fn get_fees(&self, input: GetFees) -> Box<Future<Item = Fees, Error = Error> + Send> {
        let client = self.clone();
        let url = format!("/fees");
        Box::new(
            serde_json::to_string(&input)
                .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => input))
                .into_future()
                .and_then(move |body| client.exec_query::<Fees>(&url, Some(body), Method::POST)),
        )
    }
}

#[derive(Default)]
pub struct TransactionsClientMock;

impl TransactionsClient for TransactionsClientMock {
    fn create_account(&self, _input: CreateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        Box::new(Ok(AccountResponse::default()).into_future())
    }
    fn update_account(&self, _account_id: AccountId, _payload: UpdateAccount) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        Box::new(Ok(AccountResponse::default()).into_future())
    }
    fn delete_account(&self, _account_id: AccountId) -> Box<Future<Item = AccountResponse, Error = Error> + Send> {
        Box::new(Ok(AccountResponse::default()).into_future())
    }
    fn get_account_balance(&self, _account_id: AccountId) -> Box<Future<Item = BalanceResponse, Error = Error> + Send> {
        Box::new(Ok(BalanceResponse::default()).into_future())
    }
    fn create_transaction(&self, _input: CreateTransaction) -> Box<Future<Item = TransactionResponse, Error = Error> + Send> {
        Box::new(Ok(TransactionResponse::default()).into_future())
    }
    fn get_account_transactions(
        &self,
        _account_id: AccountId,
        _offset: i64,
        _limit: i64,
    ) -> Box<Future<Item = Vec<TransactionResponse>, Error = Error> + Send> {
        Box::new(Ok(vec![TransactionResponse::default()]).into_future())
    }
    fn get_rate(&self, _input: GetRate) -> Box<Future<Item = Rate, Error = Error> + Send> {
        Box::new(Ok(Rate::default()).into_future())
    }
    fn get_fees(&self, _input: GetFees) -> Box<Future<Item = Fees, Error = Error> + Send> {
        Box::new(Ok(Fees::default()).into_future())
    }
}
