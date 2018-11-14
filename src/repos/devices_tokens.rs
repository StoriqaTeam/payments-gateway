use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::devices_tokens::dsl::*;

pub trait DeviceTokensRepo: Send + Sync + 'static {
    fn create(&self, payload: NewDeviceToken) -> RepoResult<DeviceToken>;
    fn delete(&self, id_arg: DeviceConfirmToken) -> RepoResult<DeviceToken>;
}

#[derive(Clone, Default)]
pub struct DeviceTokensRepoImpl;

impl<'a> DeviceTokensRepo for DeviceTokensRepoImpl {
    fn create(&self, payload: NewDeviceToken) -> RepoResult<DeviceToken> {
        with_tls_connection(|conn| {
            diesel::insert_into(devices_tokens)
                .values(payload.clone())
                .get_result::<DeviceToken>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }
    fn delete(&self, id_arg: DeviceConfirmToken) -> RepoResult<DeviceToken> {
        with_tls_connection(|conn| {
            let filtered = devices_tokens.filter(id.eq(id_arg));
            diesel::delete(filtered).get_result(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => id_arg)
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
    fn devices_tokens_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let devices_tokens_repo = DeviceTokensRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_device = NewDeviceToken::default();
            let res = devices_tokens_repo.create(new_device);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn devices_tokens_delete() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let devices_tokens_repo = DeviceTokensRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_device = NewDeviceToken::default();
            let device = devices_tokens_repo.create(new_device).unwrap();
            let res = devices_tokens_repo.delete(device.id);
            assert!(res.is_ok());
            res
        }));
    }
}
