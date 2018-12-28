use chrono::NaiveDateTime;
use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::users::dsl::*;

pub trait UsersRepo: Send + Sync + 'static {
    fn create(&self, payload: NewUserDB) -> RepoResult<UserDB>;
    fn update(&self, user_id: UserId, payload: UpdateUser) -> RepoResult<UserDB>;
    fn get(&self, user_id: UserId) -> RepoResult<Option<UserDB>>;
    fn get_by_email(&self, email: String) -> RepoResult<Option<UserDB>>;
    fn revoke_tokens(&self, user_id: UserId, revoke_before: NaiveDateTime) -> RepoResult<()>;
}

#[derive(Clone, Default)]
pub struct UsersRepoImpl;

impl<'a> UsersRepo for UsersRepoImpl {
    fn create(&self, payload: NewUserDB) -> RepoResult<UserDB> {
        with_tls_connection(|conn| {
            diesel::insert_into(users)
                .values(payload.clone())
                .get_result::<UserDB>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }
    fn update(&self, user_id: UserId, payload: UpdateUser) -> RepoResult<UserDB> {
        with_tls_connection(|conn| {
            let filter = users.filter(id.eq(user_id));
            let query = diesel::update(filter).set(&payload);
            query.get_result(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => user_id)
            })
        })
    }
    fn get(&self, user_id_arg: UserId) -> RepoResult<Option<UserDB>> {
        with_tls_connection(|conn| {
            users
                .filter(id.eq(user_id_arg))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => user_id_arg)
                })
        })
    }
    fn get_by_email(&self, email_: String) -> RepoResult<Option<UserDB>> {
        with_tls_connection(|conn| {
            users
                .filter(email.eq(email_.clone()))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => email_)
                })
        })
    }
    fn revoke_tokens(&self, user_id: UserId, revoke_before_: NaiveDateTime) -> RepoResult<()> {
        with_tls_connection(|conn| {
            let filter = users.filter(id.eq(user_id));
            let query = diesel::update(filter).set(revoke_before.eq(revoke_before_));
            query
                .get_result(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => user_id)
                })
                .map(|_: UserDB| ())
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
    fn users_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let users_repo = UsersRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_user = NewUserDB::default();
            let res = users_repo.create(new_user);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn users_read() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let users_repo = UsersRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_user = NewUserDB::default();
            let user = users_repo.create(new_user).unwrap();
            let res = users_repo.get(user.id);
            assert!(res.is_ok());
            res
        }));
    }
}
