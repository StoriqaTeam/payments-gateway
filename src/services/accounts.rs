use std::sync::Arc;

use futures::future::{self, Either};
use futures::stream::iter_ok;
use futures::IntoFuture;
use validator::Validate;

use super::auth::AuthService;
use super::error::*;
use client::TransactionsClient;
use models::*;
use prelude::*;
use repos::{AccountsRepo, DbExecutor};

#[derive(Clone)]
pub struct AccountsServiceImpl<E: DbExecutor> {
    auth_service: Arc<dyn AuthService>,
    accounts_repo: Arc<dyn AccountsRepo>,
    db_executor: E,
    transactions_client: Arc<dyn TransactionsClient>,
}

impl<E: DbExecutor> AccountsServiceImpl<E> {
    pub fn new(
        auth_service: Arc<AuthService>,
        accounts_repo: Arc<AccountsRepo>,
        db_executor: E,
        transactions_client: Arc<dyn TransactionsClient>,
    ) -> Self {
        Self {
            auth_service,
            accounts_repo,
            db_executor,
            transactions_client,
        }
    }
}

pub trait AccountsService: Send + Sync + 'static {
    fn create_account(
        &self,
        token: AuthenticationToken,
        user_id: UserId,
        input: CreateAccount,
    ) -> Box<Future<Item = Account, Error = Error> + Send>;
    fn get_account(&self, token: AuthenticationToken, account_id: AccountId) -> Box<Future<Item = Option<Account>, Error = Error> + Send>;
    fn update_account(
        &self,
        token: AuthenticationToken,
        account_id: AccountId,
        payload: UpdateAccount,
    ) -> Box<Future<Item = Account, Error = Error> + Send>;
    fn delete_account(&self, token: AuthenticationToken, account_id: AccountId) -> Box<Future<Item = Account, Error = Error> + Send>;
    fn get_accounts_for_user(
        &self,
        token: AuthenticationToken,
        user_id: UserId,
        offset: AccountId,
        limit: i64,
    ) -> Box<Future<Item = Vec<Account>, Error = Error> + Send>;
}

impl<E: DbExecutor> AccountsService for AccountsServiceImpl<E> {
    fn create_account(
        &self,
        token: AuthenticationToken,
        user_id: UserId,
        input: CreateAccount,
    ) -> Box<Future<Item = Account, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let transactions_client = self.transactions_client.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |auth| {
            if user_id != auth.user_id {
                Either::A(future::err(
                    ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user_id),
                ))
            } else {
                Either::B(
                    input
                        .validate()
                        .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(e) => input))
                        .into_future()
                        .and_then({
                            let input = input.clone();
                            move |_| {
                                transactions_client
                                    .create_account(input.clone())
                                    .map_err(ectx!(convert => input))
                                    .map(|acc| acc.address)
                            }
                        }).and_then(move |account_address| {
                            db_executor.execute(move || {
                                let new_account: NewAccount = (input, account_address).into();
                                accounts_repo.create(new_account.clone()).map_err(ectx!(convert => new_account))
                            })
                        }),
                )
            }
        }))
    }
    fn get_account(&self, token: AuthenticationToken, account_id: AccountId) -> Box<Future<Item = Option<Account>, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let transactions_client = self.transactions_client.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |auth| {
            db_executor
                .execute(move || {
                    let account = accounts_repo
                        .get(account_id)
                        .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                    if let Some(ref account) = account {
                        if account.user_id != auth.user_id {
                            return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => account.user_id));
                        }
                    }
                    Ok(account)
                }).and_then(move |mut account| {
                    if let Some(account) = account {
                        Either::A(
                            transactions_client
                                .get_account_balance(account.id)
                                .map_err(ectx!(convert => account.id))
                                .map(|balance| {
                                    account.balance = balance.balance;
                                    Some(account)
                                }),
                        )
                    } else {
                        Either::B(future::ok(None))
                    }
                })
        }))
    }
    fn update_account(
        &self,
        token: AuthenticationToken,
        account_id: AccountId,
        payload: UpdateAccount,
    ) -> Box<Future<Item = Account, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let auth_service = self.auth_service.clone();
        let transactions_client = self.transactions_client.clone();
        Box::new(
            payload
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(e) => payload))
                .into_future()
                .and_then(move |_| {
                    auth_service.authenticate(token).and_then(move |auth| {
                        db_executor.execute_transaction(move || {
                            let account = accounts_repo
                                .update(account_id, payload.clone())
                                .map_err(ectx!(try ErrorKind::Internal => account_id, payload))?;
                            if account.user_id != auth.user_id {
                                return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => account.user_id));
                            }
                            Ok(account)
                        })
                    })
                }).and_then(move |mut account| {
                    transactions_client
                        .get_account_balance(account.id)
                        .map_err(ectx!(convert => account.id))
                        .map(|balance| {
                            account.balance = balance.balance;
                            account
                        })
                }),
        )
    }
    fn delete_account(&self, token: AuthenticationToken, account_id: AccountId) -> Box<Future<Item = Account, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |auth| {
            db_executor.execute_transaction(move || {
                let account = accounts_repo
                    .delete(account_id)
                    .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                if account.user_id != auth.user_id {
                    return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => account.user_id));
                }
                Ok(account)
            })
        }))
    }
    fn get_accounts_for_user(
        &self,
        token: AuthenticationToken,
        user_id: UserId,
        offset: AccountId,
        limit: i64,
    ) -> Box<Future<Item = Vec<Account>, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let transactions_client = self.transactions_client.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |auth| {
            db_executor
                .execute(move || {
                    if user_id != auth.user_id {
                        return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user_id));
                    }
                    accounts_repo
                        .list_for_user(user_id, offset, limit)
                        .map_err(ectx!(ErrorKind::Internal => user_id, offset, limit))
                }).and_then(move |accounts| {
                    iter_ok::<_, Error>(accounts).fold(vec![], move |mut accounts, account| {
                        transactions_client
                            .get_account_balance(account.id)
                            .map_err(ectx!(convert => account.id))
                            .map(|balance| {
                                account.balance = balance.balance;
                                accounts.push(account);
                                accounts
                            })
                    })
                })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::*;
    use repos::*;
    use services::*;
    use tokio_core::reactor::Core;

    fn create_account_service(token: AuthenticationToken, user_id: UserId) -> AccountsServiceImpl<DbExecutorMock> {
        let auth_service = Arc::new(AuthServiceMock::new(vec![(token, user_id)]));
        let accounts_repo = Arc::new(AccountsRepoMock::default());
        let transactions_client = Arc::new(KeysClientMock::default());
        let db_executor = DbExecutorMock::default();
        AccountsServiceImpl::new(auth_service, accounts_repo, db_executor, transactions_client)
    }

    #[test]
    fn test_account_create() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let service = create_account_service(token.clone(), user_id);

        let mut new_account = CreateAccount::default();
        new_account.name = "test test test acc".to_string();
        new_account.user_id = user_id;

        let account = core.run(service.create_account(token, user_id, new_account));
        assert!(account.is_ok());
    }
    #[test]
    fn test_account_get() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let service = create_account_service(token.clone(), user_id);

        let mut new_account = CreateAccount::default();
        new_account.name = "test test test acc".to_string();
        new_account.user_id = user_id;

        let account = core.run(service.get_account(token, new_account.id));
        assert!(account.is_ok());
    }
    #[test]
    fn test_account_update() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let service = create_account_service(token.clone(), user_id);

        let mut new_account = CreateAccount::default();
        new_account.name = "test test test acc".to_string();
        new_account.user_id = user_id;

        core.run(service.create_account(token.clone(), user_id, new_account.clone()))
            .unwrap();

        let mut payload = UpdateAccount::default();
        payload.name = Some("test test test 2acc".to_string());
        let account = core.run(service.update_account(token, new_account.id, payload));
        assert!(account.is_ok());
    }
    #[test]
    fn test_account_delete() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let service = create_account_service(token.clone(), user_id);

        let mut new_account = CreateAccount::default();
        new_account.name = "test test test acc".to_string();
        new_account.user_id = user_id;
        core.run(service.create_account(token.clone(), user_id, new_account.clone()))
            .unwrap();

        let account = core.run(service.delete_account(token, new_account.id));
        assert!(account.is_ok());
    }
    #[test]
    fn test_account_get_for_users() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let service = create_account_service(token.clone(), user_id);

        let mut new_account = CreateAccount::default();
        new_account.name = "test test test acc".to_string();
        new_account.user_id = user_id;

        let account = core.run(service.get_accounts_for_user(token, new_account.user_id, new_account.id, 10));
        assert!(account.is_ok());
    }
}
