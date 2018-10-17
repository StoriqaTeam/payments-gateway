use std::sync::Arc;

use futures::future::Either;
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
use utils::log_and_capture_error;

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
        let accounts_repo = self.accounts_repo.clone();
        let db_executor = self.db_executor.clone();
        let service = self.clone();
        Box::new(self.auth_service.authenticate(token.clone()).and_then(move |user| {
            input
                .validate()
                .map_err(|e| ectx!(err e.clone(), ErrorKind::InvalidInput(e) => input))
                .into_future()
                .and_then({
                    let input = input.clone();
                    move |_| {
                        db_executor.execute(move || {
                            // check that dr account exists and it is belonging to one user
                            let dr_account_id = input.dr_account_id;
                            let dr_acc = accounts_repo
                                .get(dr_account_id)
                                .map_err(ectx!(try ErrorKind::Internal => dr_account_id))?;
                            let dr_acc =
                                dr_acc.ok_or_else(|| ectx!(try err ErrorContext::NoAccount, ErrorKind::NotFound => dr_account_id))?;
                            if dr_acc.user_id != user.id {
                                return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user.id));
                            }

                            // check that cr account exists and it is belonging to one user
                            let to = input.to.clone();
                            let input_type = input.to_type.clone();
                            match input_type {
                                ReceiptType::Account => {
                                    let cr_account_id = to.clone().to_account_id().map_err(
                                        move |_| ectx!(try err ErrorKind::MalformedInput, ErrorKind::MalformedInput => to, input_type),
                                    )?;
                                    let cr_acc = accounts_repo
                                        .get(cr_account_id)
                                        .map_err(ectx!(try ErrorKind::Internal => cr_account_id))?;
                                    let cr_acc = cr_acc
                                        .ok_or_else(|| ectx!(try err ErrorContext::NoAccount, ErrorKind::NotFound => cr_account_id))?;
                                    if cr_acc.user_id != user.id {
                                        return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user.id));
                                    }
                                    Ok((dr_acc, CrReceiptType::Account(cr_acc)))
                                }
                                ReceiptType::Address => {
                                    let cr_account_address = to.to_account_address();
                                    let cr_account_address_clone = cr_account_address.clone();
                                    let cr_acc = accounts_repo
                                        .get_by_address(cr_account_address.clone(), AccountKind::Cr)
                                        .map_err(ectx!(try ErrorKind::Internal => cr_account_address))?;
                                    if let Some(cr_acc) = cr_acc {
                                        if cr_acc.user_id != user.id {
                                            return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user.id));
                                        }
                                        Ok((dr_acc, CrReceiptType::Account(cr_acc)))
                                    } else {
                                        Ok((dr_acc, CrReceiptType::Address(cr_account_address_clone)))
                                    }
                                }
                            }
                        })
                    }
                }).and_then(move |(dr_acc, cr_acc)| match cr_acc {
                    CrReceiptType::Account(cr_acc) => Either::A(
                        service
                            .create_transaction_local(CreateTransactionLocal::new(&input, dr_acc, cr_acc))
                            .map(|tr| vec![tr]),
                    ),
                    CrReceiptType::Address(cr_account_address) => {
                        Either::B(service.withdraw(token, Withdraw::new(&input, dr_acc, cr_account_address)))
                    }
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
        let db_executor = self.db_executor.clone();
        Box::new(self.auth_service.authenticate(token).and_then(move |user| {
            db_executor.execute(move || {
                if user_id != user.id {
                    return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user.id));
                }
                transactions_repo
                    .list_for_user(user_id, offset, limit)
                    .map_err(ectx!(convert => user_id, offset, limit))
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
        Box::new(self.auth_service.authenticate(token).and_then(move |user| {
            db_executor.execute(move || {
                let account = accounts_repo
                    .get(account_id)
                    .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                if let Some(ref account) = account {
                    if account.user_id != user.id {
                        return Err(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized => user.id));
                    }
                } else {
                    return Err(ectx!(err ErrorContext::NoAccount, ErrorKind::NotFound => account_id));
                }
                transactions_repo.list_for_account(account_id).map_err(ectx!(convert => account_id))
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
        let transactions_repo = Arc::new(TransactionsRepoMock::default());
        let keys_client = Arc::new(KeysClientMock::default());
        let blockchain_client = Arc::new(BlockchainClientMock::default());
        let db_executor = DbExecutorMock::default();
        let acc_service = AccountsServiceImpl::new(
            auth_service.clone(),
            accounts_repo.clone(),
            db_executor.clone(),
            keys_client.clone(),
        );
        let trans_service = TransactionsServiceImpl::new(
            auth_service,
            transactions_repo,
            accounts_repo,
            db_executor,
            keys_client,
            blockchain_client,
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
        let dr_account = core.run(acc_service.create_account(token.clone(), user_id, dr_account)).unwrap();

        let mut new_transaction = DepositFounds::default();
        new_transaction.value = Amount::new(100501);
        new_transaction.address = dr_account.address.clone();

        core.run(trans_service.deposit_funds(token.clone(), new_transaction)).unwrap();

;        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;
        let cr_account = core
            .run(acc_service.create_account(token.clone(), user_id, cr_account.clone()))
            .unwrap();

        let mut new_transaction = CreateTransactionLocal::default();
        new_transaction.value = Amount::new(100500);
        new_transaction.cr_account = cr_account;
        new_transaction.dr_account = dr_account;

        let transaction = core.run(trans_service.create_transaction_local(new_transaction));
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

        let cr_account = core.run(acc_service.create_account(token.clone(), user_id, cr_account)).unwrap();

        let mut new_transaction = DepositFounds::default();
        new_transaction.value = Amount::new(100500);
        new_transaction.address = cr_account.address;
        new_transaction.user_id = user_id;

        let transaction = core.run(trans_service.deposit_funds(token.clone(), new_transaction)).unwrap();

        let transactions = core.run(trans_service.get_transactions_for_user(token, user_id, transaction.id, 10));
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

        let mut new_transaction = DepositFounds::default();
        new_transaction.value = Amount::new(100501);
        new_transaction.address = dr_account.address.clone();

        core.run(trans_service.deposit_funds(token.clone(), new_transaction)).unwrap();

        let mut cr_account = CreateAccount::default();
        cr_account.name = "test test test acc".to_string();
        cr_account.user_id = user_id;
        let cr_account = core.run(acc_service.create_account(token.clone(), user_id, cr_account)).unwrap();

        let mut new_transaction = CreateTransactionLocal::default();
        new_transaction.value = Amount::new(100500);
        new_transaction.cr_account = cr_account;
        new_transaction.dr_account = dr_account;

        let transaction = core.run(trans_service.create_transaction_local(new_transaction)).unwrap();
        let transaction = core.run(trans_service.get_account_transactions(token, transaction.cr_account_id));
        assert!(transaction.is_ok());
    }
}
