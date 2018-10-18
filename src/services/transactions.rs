use std::sync::Arc;

use futures::prelude::*;
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
pub struct TransactionsServiceImpl<E: DbExecutor> {
    auth_service: Arc<dyn AuthService>,
    accounts_repo: Arc<dyn AccountsRepo>,
    db_executor: E,
    transactions_client: Arc<dyn TransactionsClient>,
}

impl<E: DbExecutor> TransactionsServiceImpl<E> {
    pub fn new(
        auth_service: Arc<AuthService>,
        accounts_repo: Arc<dyn AccountsRepo>,
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

pub trait TransactionsService: Send + Sync + 'static {
    fn create_transaction(
        &self,
        token: AuthenticationToken,
        input: CreateTransaction,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send>;
    fn get_transactions_for_user(
        &self,
        token: AuthenticationToken,
        user_id: UserId,
        offset: TransactionId,
        limit: i64,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send>;
    fn get_account_transactions(
        &self,
        token: AuthenticationToken,
        account_id: AccountId,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send>;
}

impl<E: DbExecutor> TransactionsService for TransactionsServiceImpl<E> {
    fn create_transaction(
        &self,
        token: AuthenticationToken,
        input: CreateTransaction,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send> {
        let db_executor = self.db_executor.clone();
        let accounts_repo = self.accounts_repo.clone();
        let transactions_client = self.transactions_client.clone();
        Box::new(self.auth_service.authenticate(token.clone()).and_then(move |auth| {
            db_executor
                .execute({
                    let input = input.clone();
                    move || {
                        let user_id = auth.user_id;
                        let accounts = accounts_repo
                            .get_by_user(user_id)
                            .map_err(ectx!(try ErrorKind::Internal => user_id))?;
                        if !accounts.iter().any(|account| input.from == account.id) {
                            Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user_id))
                        } else {
                            Ok(())
                        }
                    }
                }).and_then(move |_| {
                    input
                        .validate()
                        .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(e) => input))
                        .into_future()
                        .and_then(move |_| {
                            transactions_client
                                .create_transaction(input.clone())
                                .map_err(ectx!(convert => input))
                                .map(|resp| resp.into_iter().map(From::from).collect())
                        })
                })
        }))
    }

    fn get_transactions_for_user(
        &self,
        token: AuthenticationToken,
        user_id: UserId,
        offset: TransactionId,
        limit: i64,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send> {
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
                        .get_by_user(user_id)
                        .map_err(ectx!(ErrorKind::Internal => user_id, offset, limit))
                }).and_then(move |accounts| {
                    iter_ok::<_, Error>(accounts).fold(vec![], move |mut total_transactions, account| {
                        transactions_client
                            .get_account_transactions(account.id)
                            .map_err(ectx!(convert => account.id))
                            .map(|resp| resp.into_iter().map(From::from).collect())
                            .and_then(|mut transactions| {
                                total_transactions.append(&mut transactions);
                                Ok(total_transactions) as Result<Vec<Transaction>, Error>
                            })
                    })
                })
        }))
    }
    fn get_account_transactions(
        &self,
        token: AuthenticationToken,
        account_id: AccountId,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let transactions_client = self.transactions_client.clone();
        Box::new(self.auth_service.authenticate(token.clone()).and_then(move |auth| {
            db_executor
                .execute({
                    move || {
                        let user_id = auth.user_id;
                        let account = accounts_repo.get(account_id).map_err(ectx!(try ErrorKind::Internal => user_id))?;
                        if let Some(account) = account {
                            if account.user_id != user_id {
                                Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user_id))
                            } else {
                                Ok(())
                            }
                        } else {
                            Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::NotFound => user_id))
                        }
                    }
                }).and_then(move |_| {
                    transactions_client
                        .get_account_transactions(account_id)
                        .map_err(ectx!(convert => account_id))
                        .map(|resp| resp.into_iter().map(From::from).collect())
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

    fn create_services(
        token: AuthenticationToken,
        user_id: UserId,
    ) -> (AccountsServiceImpl<DbExecutorMock>, TransactionsServiceImpl<DbExecutorMock>) {
        let auth_service = Arc::new(AuthServiceMock::new(vec![(token, user_id)]));
        let accounts_repo = Arc::new(AccountsRepoMock::default());
        let transactions_client = Arc::new(TransactionsClientMock::default());
        let db_executor = DbExecutorMock::default();
        let acc_service = AccountsServiceImpl::new(
            auth_service.clone(),
            accounts_repo.clone(),
            db_executor.clone(),
            transactions_client.clone(),
        );
        let trans_service = TransactionsServiceImpl::new(
            auth_service.clone(),
            accounts_repo.clone(),
            db_executor.clone(),
            transactions_client.clone(),
        );
        (acc_service, trans_service)
    }

    #[test]
    fn test_transaction_create() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let (acc_service, trans_service) = create_services(token.clone(), user_id);

        let mut dr_account = CreateAccount::default();
        dr_account.name = "test test test acc".to_string();
        dr_account.user_id = user_id;
        core.run(acc_service.create_account(token.clone(), user_id, dr_account)).unwrap();

;        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;
        core.run(acc_service.create_account(token.clone(), user_id, cr_account.clone()))
            .unwrap();

        let mut new_transaction = CreateTransaction::default();
        new_transaction.value = Amount::new(100500);

        let transaction = core.run(trans_service.create_transaction(token, new_transaction));
        assert!(transaction.is_ok());
    }
    #[test]
    fn test_transaction_get_for_users() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let (acc_service, trans_service) = create_services(token.clone(), user_id);

        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;

        core.run(acc_service.create_account(token.clone(), user_id, cr_account)).unwrap();

        let transactions = core.run(trans_service.get_transactions_for_user(token, user_id, TransactionId::generate(), 10));
        assert!(transactions.is_ok());
        assert_eq!(transactions.unwrap().len(), 1);
    }
    #[test]
    fn test_transaction_get_for_account() {
        let mut core = Core::new().unwrap();
        let token = AuthenticationToken::default();
        let user_id = UserId::generate();
        let (acc_service, trans_service) = create_services(token.clone(), user_id);

        let mut dr_account = CreateAccount::default();
        dr_account.name = "test test test acc".to_string();
        dr_account.user_id = user_id;
        let dr_account = core.run(acc_service.create_account(token.clone(), user_id, dr_account)).unwrap();

        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;
        core.run(acc_service.create_account(token.clone(), user_id, cr_account)).unwrap();

        let mut new_transaction = CreateTransaction::default();
        new_transaction.value = Amount::new(100500);

        core.run(trans_service.create_transaction(token.clone(), new_transaction)).unwrap();
        let transaction = core.run(trans_service.get_account_transactions(token, dr_account.id));
        assert!(transaction.is_ok());
    }
}
