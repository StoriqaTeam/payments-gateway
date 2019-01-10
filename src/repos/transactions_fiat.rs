use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::transactions_fiat::dsl::*;

pub trait TransactionFiatRepo: Send + Sync + 'static {
    fn create(&self, payload: NewTransactionFiat) -> RepoResult<TransactionFiat>;
    fn get(&self, transaction_id: TransactionId) -> RepoResult<Option<TransactionFiat>>;
}

#[derive(Clone, Default)]
pub struct TransactionFiatRepoImpl;

impl<'a> TransactionFiatRepo for TransactionFiatRepoImpl {
    fn create(&self, payload: NewTransactionFiat) -> RepoResult<TransactionFiat> {
        with_tls_connection(|conn| {
            diesel::insert_into(transactions_fiat)
                .values(payload.clone())
                .get_result::<TransactionFiat>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }
    fn get(&self, transaction_id: TransactionId) -> RepoResult<Option<TransactionFiat>> {
        with_tls_connection(|conn| {
            transactions_fiat
                .filter(id.eq(transaction_id.clone()))
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => transaction_id)
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

    fn create_transactions_fiat() -> RepoResult<TransactionFiat> {
        let transactions_fiat_repo = TransactionFiatRepoImpl::default();
        let new_transactions_fiat = NewTransactionFiat::default();
        transactions_fiat_repo.create(new_transactions_fiat)
    }

    fn create_executor() -> DbExecutorImpl {
        let config = Config::new().unwrap();
        let manager = ConnectionManager::<PgConnection>::new(config.database.url);
        let db_pool = r2d2::Pool::builder().build(manager).unwrap();
        let cpu_pool = CpuPool::new(1);
        DbExecutorImpl::new(db_pool.clone(), cpu_pool.clone())
    }

    #[test]
    fn transactions_fiat_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let res = create_device();
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn transactions_fiat_read() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let transactions_fiat_repo = TransactionFiatRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let transactions_fiat = create_device().unwrap();
            let res = transactions_fiat_repo.get(transactions_fiat.id);
            assert!(res.is_ok());
            res
        }));
    }
}
