use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::devices_tokens::dsl::*;

pub trait DeviceTokensRepo: Send + Sync + 'static {
    fn upsert(&self, payload: NewDeviceToken) -> RepoResult<DeviceToken>;
    fn get(&self, id_arg: DeviceConfirmToken) -> RepoResult<Option<DeviceToken>>;
    fn get_by_public_key(&self, public_key_arg: DevicePublicKey) -> RepoResult<Option<DeviceToken>>;
}

#[derive(Clone, Default)]
pub struct DeviceTokensRepoImpl;

impl<'a> DeviceTokensRepo for DeviceTokensRepoImpl {
    fn upsert(&self, payload: NewDeviceToken) -> RepoResult<DeviceToken> {
        with_tls_connection(|conn| {
            let device_id_clone = payload.device_id.clone();
            let public_key_clone = payload.public_key.clone();
            let user_id_clone = payload.user_id.clone();
            let filtered = devices_tokens
                .filter(device_id.eq(device_id_clone.clone()))
                .filter(public_key.eq(public_key_clone.clone()))
                .filter(user_id.eq(user_id_clone.clone()));
            let token: Option<DeviceToken> = filtered.clone().get_result(conn).optional().map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(try err e, error_kind => device_id_clone)
            })?;

            if token.is_some() {
                diesel::update(filtered)
                    .set(updated_at.eq(::chrono::Utc::now().naive_utc()))
                    .get_result(conn)
                    .map_err(move |e| {
                        let error_kind = ErrorKind::from(&e);
                        ectx!(err e, error_kind)
                    })
            } else {
                diesel::insert_into(devices_tokens)
                    .values(payload.clone())
                    .get_result::<DeviceToken>(conn)
                    .map_err(move |e| {
                        let error_kind = ErrorKind::from(&e);
                        ectx!(err e, error_kind => payload)
                    })
            }
        })
    }
    fn get(&self, id_arg: DeviceConfirmToken) -> RepoResult<Option<DeviceToken>> {
        with_tls_connection(|conn| {
            let filtered = devices_tokens.filter(id.eq(id_arg));
            filtered.get_result(conn).optional().map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => id_arg)
            })
        })
    }
    fn get_by_public_key(&self, public_key_arg: DevicePublicKey) -> RepoResult<Option<DeviceToken>> {
        with_tls_connection(|conn| {
            let filtered = devices_tokens.filter(public_key.eq(public_key_arg.clone()));
            filtered.get_result(conn).optional().map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => public_key_arg)
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

    fn get_or_create_user() -> UserId {
        let users_repo = UsersRepoImpl::default();
        users_repo
            .get(UserId::new(1))
            .unwrap()
            .unwrap_or_else(|| {
                let new_user = NewUserDB {
                    id: UserId::new(1),
                    email: "test_user_noreply@storiqa.com".to_string(),
                    first_name: "FirstName".to_string(),
                    last_name: "LastName".to_string(),
                    phone: None,
                    device_id: None,
                    device_os: None,
                };
                users_repo.create(new_user).unwrap()
            })
            .id
    }

    fn upsert_device_token() -> RepoResult<DeviceToken> {
        let user_id_ = get_or_create_user();
        let devices_tokens_repo = DeviceTokensRepoImpl::default();
        let new_device_token = NewDeviceToken {
            user_id: user_id_,
            ..NewDeviceToken::default()
        };
        devices_tokens_repo.upsert(new_device_token)
    }

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
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let res = upsert_device_token();
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
            let device = upsert_device_token().unwrap();
            let res = devices_tokens_repo.get(device.id);
            assert!(res.is_ok());
            res
        }));
    }
}
