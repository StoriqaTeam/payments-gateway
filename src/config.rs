use std::env;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};
use logger::{FileLogConfig, GrayLogConfig};
use models::*;
use sentry_integration::SentryConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub client: Client,
    pub auth: Auth,
    pub cpu_pool: CpuPool,
    pub sentry: Option<SentryConfig>,
    pub graylog: Option<GrayLogConfig>,
    pub filelog: Option<FileLogConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub dns_threads: usize,
    pub storiqa_url: String,
    pub transactions_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Auth {
    pub storiqa_jwt_public_key_base64: String,
    pub storiqa_jwt_valid_secs: usize,
    pub storiqa_transactions_token: AuthenticationToken,
    pub storiqa_transactions_user_id: WorkspaceId,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CpuPool {
    pub size: usize,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Merge development.toml if RUN_MODE variable is not set
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;
        s.merge(File::with_name("config/secret.toml").required(false))?;

        s.merge(Environment::with_prefix("STQ_PAYMENTS"))?;
        s.try_into()
    }
}
