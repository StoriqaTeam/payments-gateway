use chrono::NaiveDateTime;
use std::sync::{Arc, Mutex};

use super::accounts::*;
use super::error::*;
use super::executor::DbExecutor;
use super::types::RepoResult;
use super::users::*;
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
            })
            .nth(0)
            .cloned();
        Ok(u.unwrap())
    }
    fn delete(&self, account_id: AccountId) -> RepoResult<Account> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.id == account_id).nth(0).cloned().unwrap())
    }
    fn list_for_user(&self, user_id_arg: UserId, _offset: i64, _limit: i64) -> RepoResult<Vec<Account>> {
        let data = self.data.lock().unwrap();
        Ok(data.clone().into_iter().filter(|x| x.user_id == user_id_arg).collect())
    }
    fn get_by_user(&self, user_id_arg: UserId) -> RepoResult<Vec<Account>> {
        let data = self.data.lock().unwrap();
        Ok(data.clone().into_iter().filter(|x| x.user_id == user_id_arg).collect())
    }
}

#[derive(Clone, Default)]
pub struct UsersRepoMock {
    data: Arc<Mutex<Vec<UserDB>>>,
}

impl UsersRepo for UsersRepoMock {
    fn create(&self, payload: NewUserDB) -> Result<UserDB, Error> {
        let mut data = self.data.lock().unwrap();
        let res: UserDB = payload.into();
        data.push(res.clone());
        Ok(res)
    }
    fn update(&self, user_id: UserId, _payload: UpdateUser) -> RepoResult<UserDB> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.id == user_id).nth(0).cloned().unwrap())
    }
    fn get(&self, user_id: UserId) -> RepoResult<Option<UserDB>> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.id == user_id).nth(0).cloned())
    }
    fn get_by_email(&self, email_: String) -> RepoResult<Option<UserDB>> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.email == email_).nth(0).cloned())
    }
    fn revoke_tokens(&self, _user_id: UserId, _revoke_before: NaiveDateTime) -> RepoResult<()> {
        Ok(())
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
