use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::accounts::dsl::*;

pub trait AccountsRepo: Send + Sync + 'static {
    fn create(&self, payload: NewAccount) -> RepoResult<Account>;
    fn get(&self, account_id: AccountId) -> RepoResult<Option<Account>>;
    fn update(&self, account_id: AccountId, payload: UpdateAccount) -> RepoResult<Account>;
    fn delete(&self, account_id: AccountId) -> RepoResult<Account>;
    fn list_for_user(&self, user_id_arg: UserId, offset: i64, limit: i64) -> RepoResult<Vec<Account>>;
    fn get_by_user(&self, user_id_arg: UserId) -> RepoResult<Vec<Account>>;
}

#[derive(Clone, Default)]
pub struct AccountsRepoImpl;

impl<'a> AccountsRepo for AccountsRepoImpl {
    fn create(&self, payload: NewAccount) -> RepoResult<Account> {
        with_tls_connection(|conn| {
            diesel::insert_into(accounts)
                .values(payload.clone())
                .get_result::<Account>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }
    fn get(&self, account_id_arg: AccountId) -> RepoResult<Option<Account>> {
        with_tls_connection(|conn| {
            accounts
                .filter(id.eq(account_id_arg))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => account_id_arg)
                })
        })
    }
    fn update(&self, account_id_arg: AccountId, payload: UpdateAccount) -> RepoResult<Account> {
        with_tls_connection(|conn| {
            let f = accounts.filter(id.eq(account_id_arg));
            diesel::update(f).set(payload.clone()).get_result(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => account_id_arg, payload)
            })
        })
    }
    fn delete(&self, account_id_arg: AccountId) -> RepoResult<Account> {
        with_tls_connection(|conn| {
            let filtered = accounts.filter(id.eq(account_id_arg));
            diesel::delete(filtered).get_result(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => account_id_arg)
            })
        })
    }
    fn list_for_user(&self, user_id_arg: UserId, offset: i64, limit: i64) -> RepoResult<Vec<Account>> {
        with_tls_connection(|conn| {
            let mut query = accounts.filter(user_id.eq(user_id_arg)).order(id).offset(offset).limit(limit);

            query.get_results(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => user_id_arg, offset, limit)
            })
        })
    }
    fn get_by_user(&self, user_id_arg: UserId) -> RepoResult<Vec<Account>> {
        with_tls_connection(|conn| {
            let query = accounts.filter(user_id.eq(user_id_arg)).order(id);
            query.get_results(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => user_id_arg)
            })
        })
    }
}

#[cfg(test)]
pub mod tests {
    use diesel::r2d2::ConnectionManager;
    use diesel::PgConnection;
    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Core;

    use super::*;
    use config::Config;
    use repos::DbExecutorImpl;

    fn create_executor() -> DbExecutorImpl {
        let config = Config::new().unwrap();
        let manager = ConnectionManager::<PgConnection>::new(config.database.url);
        let db_pool = r2d2::Pool::builder().build(manager).unwrap();
        let cpu_pool = CpuPool::new(1);
        DbExecutorImpl::new(db_pool.clone(), cpu_pool.clone())
    }

    #[test]
    fn accounts_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let accounts_repo = AccountsRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_account = NewAccount::default();
            let res = accounts_repo.create(new_account);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn accounts_read() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let accounts_repo = AccountsRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_account = NewAccount::default();
            let account = accounts_repo.create(new_account).unwrap();
            let res = accounts_repo.get(account.id);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn accounts_update() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let accounts_repo = AccountsRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_account = NewAccount::default();
            let account = accounts_repo.create(new_account).unwrap();

            let payload = UpdateAccount {
                name: "test".to_string(),
                ..Default::default()
            };
            let res = accounts_repo.update(account.id, payload);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn accounts_delete() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let accounts_repo = AccountsRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_account = NewAccount::default();
            let account = accounts_repo.create(new_account).unwrap();
            let res = accounts_repo.delete(account.id);
            assert!(res.is_ok());
            res
        }));
    }
    #[test]
    fn accounts_list() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let accounts_repo = AccountsRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_account = NewAccount::default();
            let account = accounts_repo.create(new_account).unwrap();
            let res = accounts_repo.list_for_user(account.user_id, 0, 1);
            assert!(res.is_ok());
            res
        }));
    }
    #[test]
    fn accounts_get_by_user() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let accounts_repo = AccountsRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_account = NewAccount::default();
            let account = accounts_repo.create(new_account).unwrap();
            let res = accounts_repo.get_by_user(account.user_id);
            assert!(res.is_ok());
            res
        }));
    }
}
