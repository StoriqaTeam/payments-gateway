use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::devices::dsl::*;

pub trait DevicesRepo: Send + Sync + 'static {
    fn create(&self, payload: NewDevice) -> RepoResult<Device>;
    fn update_timestamp(&self, device_id_arg: DeviceId, user_id_arg: UserId, new_timestamp: i64) -> RepoResult<Device>;
    fn get(&self, device_id_arg: DeviceId, user_id_arg: UserId) -> RepoResult<Option<Device>>;
    fn get_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<Device>>;
}

#[derive(Clone, Default)]
pub struct DevicesRepoImpl;

impl<'a> DevicesRepo for DevicesRepoImpl {
    fn create(&self, payload: NewDevice) -> RepoResult<Device> {
        with_tls_connection(|conn| {
            diesel::insert_into(devices)
                .values(payload.clone())
                .get_result::<Device>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }
    fn update_timestamp(&self, device_id_arg: DeviceId, user_id_arg: UserId, new_timestamp: i64) -> RepoResult<Device> {
        with_tls_connection(|conn| {
            let f = devices.filter(device_id.eq(device_id_arg.clone())).filter(user_id.eq(user_id_arg));
            diesel::update(f)
                .set(last_timestamp.eq(new_timestamp))
                .get_result::<Device>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => device_id_arg, user_id_arg, new_timestamp)
                })
        })
    }
    fn get(&self, device_id_arg: DeviceId, user_id_arg: UserId) -> RepoResult<Option<Device>> {
        with_tls_connection(|conn| {
            devices
                .filter(device_id.eq(device_id_arg.clone()))
                .filter(user_id.eq(user_id_arg))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => device_id_arg, user_id_arg)
                })
        })
    }
    fn get_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<Device>> {
        with_tls_connection(|conn| {
            devices.filter(user_id.eq(user_id_arg)).get_results(conn).map_err(move |e| {
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
    fn devices_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let devices_repo = DevicesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_device = NewDevice::default();
            let res = devices_repo.create(new_device);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn devices_read() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let devices_repo = DevicesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_device = NewDevice::default();
            let device = devices_repo.create(new_device).unwrap();
            let res = devices_repo.get(device.device_id, device.user_id);
            assert!(res.is_ok());
            res
        }));
    }
}
