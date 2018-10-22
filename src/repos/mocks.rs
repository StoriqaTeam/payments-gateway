use std::sync::{Arc, Mutex};

use super::accounts::*;
use super::error::*;
use super::executor::DbExecutor;
use super::types::RepoResult;
use models::*;
use prelude::*;

#[derive(Clone, Default)]
pub struct AccountsRepoMock {
    data: Arc<Mutex<Vec<Account>>>,
}

impl AccountsRepo for AccountsRepoMock {
    fn create(&self, payload: NewAccount) -> Result<Account, Error> {
        let mut data = self.data.lock().unwrap();
        let res: Account = payload.into();
        data.push(res.clone());
        Ok(res)
    }
    fn get(&self, account_id: AccountId) -> RepoResult<Option<Account>> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.id == account_id).nth(0).cloned())
    }
    fn update(&self, account_id: AccountId, payload: UpdateAccount) -> RepoResult<Account> {
        let mut data = self.data.lock().unwrap();
        let u = data
            .iter_mut()
            .filter_map(|x| {
                if x.id == account_id {
                    x.name = payload.name.clone();
                    Some(x)
                } else {
                    None
                }
            }).nth(0)
            .cloned();
        Ok(u.unwrap())
    }
    fn delete(&self, account_id: AccountId) -> RepoResult<Account> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.id == account_id).nth(0).cloned().unwrap())
    }
    fn list_for_user(&self, user_id_arg: UserId, _offset: Option<AccountId>, _limit: Option<i64>) -> RepoResult<Vec<Account>> {
        let data = self.data.lock().unwrap();
        Ok(data.clone().into_iter().filter(|x| x.user_id == user_id_arg).collect())
    }
    fn get_by_user(&self, user_id_arg: UserId) -> RepoResult<Vec<Account>> {
        let data = self.data.lock().unwrap();
        Ok(data.clone().into_iter().filter(|x| x.user_id == user_id_arg).collect())
    }
}

#[derive(Clone, Default)]
pub struct DbExecutorMock;

impl DbExecutor for DbExecutorMock {
    fn execute<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Send + 'static,
    {
        Box::new(f().into_future())
    }
    fn execute_transaction<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Send + 'static,
    {
        Box::new(f().into_future())
    }
    fn execute_test_transaction<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Fail,
    {
        Box::new(f().into_future())
    }
}
