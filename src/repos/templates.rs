use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::templates::dsl::*;

pub trait TemplatesRepo: Send + Sync + 'static {
    fn create(&self, payload: NewTemplate) -> RepoResult<Template>;
    fn get(&self, name: TemplateName) -> RepoResult<Option<Template>>;
}

#[derive(Clone, Default)]
pub struct TemplatesRepoImpl;

impl<'a> TemplatesRepo for TemplatesRepoImpl {
    fn create(&self, payload: NewTemplate) -> RepoResult<Template> {
        with_tls_connection(|conn| {
            diesel::insert_into(templates)
                .values(payload.clone())
                .get_result::<Template>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }
    fn get(&self, name_arg: TemplateName) -> RepoResult<Option<Template>> {
        with_tls_connection(|conn| {
            templates
                .filter(name.eq(name_arg.clone()))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => name_arg)
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
    fn templates_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let templates_repo = TemplatesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_template_ = NewTemplate::default();
            let res = templates_repo.create(new_template_);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn templates_read() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let templates_repo = TemplatesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_template_ = NewTemplate::default();
            let template_ = templates_repo.create(new_template_).unwrap();
            let res = templates_repo.get(template_.name);
            assert!(res.is_ok());
            res
        }));
    }
}
