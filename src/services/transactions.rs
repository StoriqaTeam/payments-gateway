use std::sync::Arc;

use futures::prelude::*;
use futures::stream::iter_ok;
use futures::IntoFuture;
use serde_json;
use validator::Validate;

use super::error::*;
use client::TransactionsClient;
use models::*;
use prelude::*;
use repos::{AccountsRepo, DbExecutor, UsersRepo};

#[derive(Clone)]
pub struct TransactionsServiceImpl<E: DbExecutor> {
    accounts_repo: Arc<dyn AccountsRepo>,
    users_repo: Arc<dyn UsersRepo>,
    db_executor: E,
    transactions_client: Arc<dyn TransactionsClient>,
}

impl<E: DbExecutor> TransactionsServiceImpl<E> {
    pub fn new(
        accounts_repo: Arc<dyn AccountsRepo>,
        users_repo: Arc<dyn UsersRepo>,
        db_executor: E,
        transactions_client: Arc<dyn TransactionsClient>,
    ) -> Self {
        Self {
            accounts_repo,
            users_repo,
            db_executor,
            transactions_client,
        }
    }
}

pub trait TransactionsService: Send + Sync + 'static {
    fn create_transaction(&self, user_id: UserId, input: CreateTransaction) -> Box<Future<Item = Transaction, Error = Error> + Send>;
    fn get_transactions_for_user(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send>;
    fn get_account_transactions(
        &self,
        user_id: UserId,
        account_id: AccountId,
        offset: i64,
        limit: i64,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send>;
    fn add_user_to_transaction(&self, transaction: Transaction) -> Box<Future<Item = Transaction, Error = Error> + Send>;
    fn get_rate(&self, rate: GetRate) -> Box<Future<Item = Rate, Error = Error> + Send>;
    fn get_fees(&self, rate: GetFees) -> Box<Future<Item = Fees, Error = Error> + Send>;
}

impl<E: DbExecutor> TransactionsService for TransactionsServiceImpl<E> {
    fn create_transaction(&self, user_id: UserId, input: CreateTransaction) -> Box<Future<Item = Transaction, Error = Error> + Send> {
        let db_executor = self.db_executor.clone();
        let accounts_repo = self.accounts_repo.clone();
        let transactions_client = self.transactions_client.clone();
        let service = self.clone();
        Box::new(
            db_executor
                .execute({
                    let input = input.clone();
                    let input_from = input.from.clone();
                    move || {
                        let accounts = accounts_repo
                            .get_by_user(user_id)
                            .map_err(ectx!(try ErrorKind::Internal => user_id))?;
                        if !accounts.iter().any(|account| input_from == account.id) {
                            Err(ectx!(err ErrorContext::NoAccount, ErrorKind::Unauthorized => user_id, input_from))
                        } else {
                            Ok(())
                        }
                    }
                })
                .and_then(move |_| {
                    input
                        .validate()
                        .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(serde_json::to_string(&e).unwrap_or_default()) => input))
                        .into_future()
                        .and_then(move |_| {
                            transactions_client
                                .create_transaction(input.clone())
                                .map_err(ectx!(convert => input))
                                .map(From::from)
                        })
                })
                .and_then(move |transaction: Transaction| service.add_user_to_transaction(transaction)),
        )
    }

    fn get_transactions_for_user(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let transactions_client = self.transactions_client.clone();
        let service = self.clone();
        Box::new(
            db_executor
                .execute(move || accounts_repo.get_by_user(user_id).map_err(ectx!(ErrorKind::Internal => user_id)))
                .and_then(move |accounts| {
                    iter_ok::<_, Error>(accounts).fold(vec![], move |mut total_transactions, account| {
                        transactions_client
                            .get_account_transactions(account.id, offset, limit)
                            .map_err(ectx!(convert => account.id, offset, limit))
                            .map(|resp| resp.into_iter().map(From::from).collect())
                            .and_then(|mut transactions| {
                                total_transactions.append(&mut transactions);
                                Ok(total_transactions) as Result<Vec<Transaction>, Error>
                            })
                    })
                })
                .and_then(move |transactions: Vec<Transaction>| {
                    iter_ok::<_, Error>(transactions).fold(vec![], move |mut transactions, transaction| {
                        service.add_user_to_transaction(transaction).and_then(|res| {
                            transactions.push(res);
                            Ok(transactions) as Result<Vec<Transaction>, Error>
                        })
                    })
                }),
        )
    }
    fn get_account_transactions(
        &self,
        user_id: UserId,
        account_id: AccountId,
        offset: i64,
        limit: i64,
    ) -> Box<Future<Item = Vec<Transaction>, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let transactions_client = self.transactions_client.clone();
        let service = self.clone();
        Box::new(
            db_executor
                .execute({
                    move || {
                        let account = accounts_repo
                            .get(account_id)
                            .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                        if let Some(account) = account {
                            if account.user_id != user_id {
                                Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user_id, account.user_id))
                            } else {
                                Ok(())
                            }
                        } else {
                            Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::NotFound => user_id))
                        }
                    }
                })
                .and_then(move |_| {
                    transactions_client
                        .get_account_transactions(account_id, offset, limit)
                        .map_err(ectx!(convert => account_id, offset, limit))
                        .map(|resp| resp.into_iter().map(From::from).collect())
                })
                .and_then(move |transactions: Vec<Transaction>| {
                    iter_ok::<_, Error>(transactions).fold(vec![], move |mut transactions, transaction| {
                        service.add_user_to_transaction(transaction).and_then(|res| {
                            transactions.push(res);
                            Ok(transactions) as Result<Vec<Transaction>, Error>
                        })
                    })
                }),
        )
    }
    fn add_user_to_transaction(&self, mut transaction: Transaction) -> Box<Future<Item = Transaction, Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let users_repo = self.users_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(db_executor.execute({
            move || {
                for from in &mut transaction.from {
                    if let Some(account_id) = from.account_id {
                        let account = accounts_repo
                            .get(account_id)
                            .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                        if let Some(account) = account {
                            let user = users_repo
                                .get(account.user_id)
                                .map_err(ectx!(try ErrorKind::Internal => account.user_id))?;
                            if let Some(user) = user {
                                from.owner_name = Some(user.get_full_name());
                            }
                        } else {
                            // if account, for example, was deleted
                            from.account_id = None;
                        }
                    }
                }
                if let Some(account_id) = transaction.to.account_id {
                    let account = accounts_repo
                        .get(account_id)
                        .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                    if let Some(account) = account {
                        let user = users_repo
                            .get(account.user_id)
                            .map_err(ectx!(try ErrorKind::Internal => account.user_id))?;
                        if let Some(user) = user {
                            transaction.to.owner_name = Some(user.get_full_name());
                        }
                    } else {
                        // if account, for example, was deleted
                        transaction.to.account_id = None;
                    }
                }
                Ok(transaction)
            }
        }))
    }

    fn get_rate(&self, rate: GetRate) -> Box<Future<Item = Rate, Error = Error> + Send> {
        let transactions_client = self.transactions_client.clone();
        Box::new(transactions_client.get_rate(rate.clone()).map_err(ectx!(convert => rate)))
    }

    fn get_fees(&self, get_fees: GetFees) -> Box<Future<Item = Fees, Error = Error> + Send> {
        let transactions_client = self.transactions_client.clone();
        Box::new(transactions_client.get_fees(get_fees.clone()).map_err(ectx!(convert => get_fees)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::*;
    use repos::*;
    use services::*;
    use tokio_core::reactor::Core;

    fn create_services() -> (AccountsServiceImpl<DbExecutorMock>, TransactionsServiceImpl<DbExecutorMock>) {
        let accounts_repo = Arc::new(AccountsRepoMock::default());
        let users_repo = Arc::new(UsersRepoMock::default());
        let transactions_client = Arc::new(TransactionsClientMock::default());
        let db_executor = DbExecutorMock::default();
        let acc_service = AccountsServiceImpl::new(accounts_repo.clone(), db_executor.clone(), transactions_client.clone());
        let trans_service = TransactionsServiceImpl::new(
            accounts_repo.clone(),
            users_repo.clone(),
            db_executor.clone(),
            transactions_client.clone(),
        );
        (acc_service, trans_service)
    }

    #[test]
    fn test_transaction_create() {
        let mut core = Core::new().unwrap();
        let user_id = UserId::generate();
        let (acc_service, trans_service) = create_services();

        let mut dr_account = CreateAccount::default();
        dr_account.name = "test test test acc".to_string();
        dr_account.user_id = user_id;
        core.run(acc_service.create_account(dr_account.clone())).unwrap();

        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;
        core.run(acc_service.create_account(cr_account.clone())).unwrap();

        let mut new_transaction = CreateTransaction::default();
        new_transaction.value = Amount::new(100500);
        new_transaction.from = dr_account.id;

        let transaction = core.run(trans_service.create_transaction(user_id, new_transaction));
        assert!(transaction.is_ok());
    }
    #[test]
    fn test_transaction_get_for_users() {
        let mut core = Core::new().unwrap();
        let user_id = UserId::generate();
        let (acc_service, trans_service) = create_services();

        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;

        core.run(acc_service.create_account(cr_account)).unwrap();

        let transactions = core.run(trans_service.get_transactions_for_user(user_id, 0, 10));
        assert!(transactions.is_ok());
        assert_eq!(transactions.unwrap().len(), 1);
    }
    #[test]
    fn test_transaction_get_for_account() {
        let mut core = Core::new().unwrap();
        let user_id = UserId::generate();
        let (acc_service, trans_service) = create_services();

        let mut dr_account = CreateAccount::default();
        dr_account.name = "test test test acc".to_string();
        dr_account.user_id = user_id;
        let dr_account = core.run(acc_service.create_account(dr_account)).unwrap();

        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;
        core.run(acc_service.create_account(cr_account)).unwrap();

        let mut new_transaction = CreateTransaction::default();
        new_transaction.value = Amount::new(100500);
        new_transaction.from = dr_account.id;

        core.run(trans_service.create_transaction(user_id, new_transaction)).unwrap();
        let transaction = core.run(trans_service.get_account_transactions(user_id, dr_account.id, 0, 10));
        assert!(transaction.is_ok());
    }
}
