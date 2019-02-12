#![allow(proc_macro_derive_resolution_fallback)]

extern crate chrono;
extern crate futures;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate futures_cpupool;
extern crate hyper;
extern crate r2d2;
extern crate serde;
extern crate serde_json;
extern crate serde_qs;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate lapin_async;
extern crate lapin_futures;
#[macro_use]
extern crate log;
extern crate config as config_crate;
#[macro_use]
extern crate http_router;
extern crate base64;
extern crate hyper_tls;
extern crate jsonwebtoken;
extern crate regex;
#[macro_use]
extern crate validator_derive;
extern crate num;
extern crate validator;
#[macro_use]
extern crate sentry;
extern crate crypto;
extern crate gelf;
extern crate handlebars;
extern crate secp256k1;
extern crate simplelog;
extern crate tokio;
extern crate tokio_core;
extern crate uuid;

#[macro_use]
mod macros;
mod api;
mod client;
mod config;
mod logger;
mod models;
mod prelude;
mod rabbit;
mod repos;
mod schema;
mod sentry_integration;
mod services;
mod utils;

use std::sync::Arc;
use std::time::{Duration, Instant};

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use futures::future::Either;
use futures_cpupool::CpuPool;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::{Delay, Timeout};

use self::models::*;
use config::Config;
use rabbit::{RabbitConnectionManager, TransactionConsumerImpl, TransactionPublisherImpl};
use repos::{AccountsRepoImpl, DbExecutorImpl, DevicesRepoImpl, UsersRepoImpl};
use services::Notificator;
use utils::log_error;

pub const DELAY_BEFORE_NACK: u64 = 1000;
pub const DELAY_BEFORE_RECONNECT: u64 = 1000;

pub fn hello() {
    println!("Hello world");
}

pub fn print_config() {
    println!("Parsed config: {:?}", get_config());
}

pub fn start_server() {
    let config = get_config();
    // Prepare sentry integration
    let _sentry = sentry_integration::init(config.sentry.as_ref());
    // Prepare logger
    logger::init(&config);

    let database_url = config.database.url.clone();
    let manager = ConnectionManager::<PgConnection>::new(database_url.clone());
    let db_pool = r2d2::Pool::builder().build(manager).unwrap();
    let cpu_pool = CpuPool::new(config.cpu_pool.size);
    let db_executor = DbExecutorImpl::new(db_pool.clone(), cpu_pool.clone());
    let config_clone = config.clone();

    let mut rt = Runtime::new().expect("Failed to create a Tokio runtime");
    debug!("Started creating rabbit connection pool");

    let rabbit_connection_manager = rt
        .block_on(RabbitConnectionManager::create(&config_clone))
        .map_err(|e| {
            log_error(&e);
        })
        .expect("Failed to create the rabbit connection manager");
    debug!("Finished creating rabbit connection pool");
    let consumer = TransactionConsumerImpl::new(rabbit_connection_manager.clone(), config_clone.auth.storiqa_transactions_user_id);
    let channel = Arc::new(rabbit_connection_manager.get_channel().expect("Can not get channel from pool"));
    let publisher = rt
        .block_on(TransactionPublisherImpl::init(channel))
        .map_err(|e| {
            log_error(&e);
        })
        .expect("Can not create publisher for transactions in rabbit");
    let publisher = Arc::new(publisher);
    let publisher_clone = publisher.clone();
    let fetcher = Notificator::new(
        Arc::new(AccountsRepoImpl),
        Arc::new(UsersRepoImpl),
        Arc::new(DevicesRepoImpl),
        db_executor.clone(),
        publisher_clone,
    );
    let (stream, channel) = rt
        .block_on(consumer.subscribe())
        .expect("Can not create subscribers for transactions in rabbit");
    debug!("Subscribing to rabbit");
    let fetcher_clone = fetcher.clone();
    let timeout = config_clone.rabbit.restart_subscription_secs as u64;
    let subscription = stream
        .for_each(move |message| {
            trace!("got message: {}", MessageDelivery::new(message.clone()));
            let delivery_tag = message.delivery_tag;
            let channel = channel.clone();
            let fetcher_future = fetcher_clone.handle_message(message.data);
            let timeout = Duration::from_secs(timeout);
            Timeout::new(fetcher_future, timeout).then(move |res| match res {
                Ok(_) => Either::A(channel.basic_ack(delivery_tag, false).map_err(|e| {
                    error!("Error sending ack: {}", e);
                    e
                })),
                Err(e) => {
                    error!("Error during message handling: {}", e);
                    Either::B(
                        Delay::new(Instant::now() + Duration::from_millis(DELAY_BEFORE_NACK)).then(move |_| {
                            channel.basic_nack(0, true, true).map_err(|e| {
                                error!("Error sending nack: {}", e);
                                e
                            })
                        }),
                    )
                }
            })
        })
        .map_err(|_| ());

    rt.spawn(subscription);
    rt.spawn(api::server(config, publisher.clone()));

    rt.shutdown_on_idle().wait().expect("Tokio runtime shutdown failed")
}

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
