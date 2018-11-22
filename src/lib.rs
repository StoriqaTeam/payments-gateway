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

use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use failure::Fail;
use futures::future::{self, Either};
use futures_cpupool::CpuPool;
use lapin_futures::channel::Channel;
use tokio::net::tcp::TcpStream;
use tokio::prelude::*;
use tokio::timer::{Delay, Timeout};

use self::models::*;
use config::Config;
use rabbit::{ConnectionHooks, R2D2ErrorHandler, RabbitConnectionManager, TransactionConsumerImpl, TransactionPublisherImpl};
use rabbit::{ErrorKind, ErrorSource};
use repos::{AccountsRepoImpl, DbExecutorImpl, DevicesRepoImpl, UsersRepoImpl};
use services::Notificator;
use utils::log_error;

pub const DELAY_BEFORE_NACK: u64 = 1000;

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

    let mut core = tokio_core::reactor::Core::new().unwrap();
    debug!("Started creating rabbit connection pool");

    let rabbit_thread_pool = futures_cpupool::CpuPool::new(config_clone.rabbit.thread_pool_size);
    let rabbit_connection_manager = core
        .run(RabbitConnectionManager::create(&config_clone))
        .map_err(|e| {
            log_error(&e);
        }).unwrap();
    let rabbit_connection_pool = r2d2::Pool::builder()
        .max_size(config_clone.rabbit.connection_pool_size as u32)
        .connection_customizer(Box::new(ConnectionHooks))
        .error_handler(Box::new(R2D2ErrorHandler))
        .build(rabbit_connection_manager)
        .expect("Cannot build rabbit connection pool");
    debug!("Finished creating rabbit connection pool");
    let consumer = TransactionConsumerImpl::new(
        rabbit_connection_pool.clone(),
        rabbit_thread_pool.clone(),
        config_clone.auth.storiqa_transactions_user_id,
    );
    let publisher = Arc::new(TransactionPublisherImpl::new(rabbit_connection_pool, rabbit_thread_pool));
    core.run(publisher.init())
        .map_err(|e| {
            log_error(&e);
        }).unwrap();
    let publisher_clone = publisher.clone();
    let fetcher = Notificator::new(
        Arc::new(AccountsRepoImpl),
        Arc::new(UsersRepoImpl),
        Arc::new(DevicesRepoImpl),
        db_executor.clone(),
        publisher_clone,
    );
    thread::spawn(move || {
        let mut core = tokio_core::reactor::Core::new().unwrap();

        loop {
            info!("Subscribing to rabbit");
            let counters = Arc::new(Mutex::new((0usize, 0usize, 0usize, 0usize, 0usize)));
            let consumers_to_close: Arc<Mutex<Vec<(Channel<TcpStream>, String)>>> = Arc::new(Mutex::new(Vec::new()));
            let consumers_to_close_clone = consumers_to_close.clone();
            let resubscribe_duration = Duration::from_secs(config_clone.rabbit.restart_subscription_secs as u64);
            let counters_clone = counters.clone();
            let fetcher_clone = fetcher.clone();
            let subscription = consumer
                .subscribe()
                .and_then(move |(stream, channel)| {
                    let counters_clone = counters.clone();
                    let consumers_to_close = consumers_to_close.clone();
                    let mut consumers_to_close_lock = consumers_to_close.lock().unwrap();
                    consumers_to_close_lock.push((channel.clone(), stream.consumer_tag.clone()));
                    drop(consumers_to_close_lock);
                    stream
                        .for_each(move |message| {
                            trace!("got message: {}", MessageDelivery::new(message.clone()));
                            let delivery_tag = message.delivery_tag;
                            let mut counters = counters_clone.lock().unwrap();
                            counters.0 += 1;
                            drop(counters);
                            let counters_clone2 = counters_clone.clone();

                            let channel = channel.clone();
                            fetcher_clone.handle_message(message.data).then(move |res| match res {
                                Ok(_) => {
                                    let counters_clone = counters_clone2.clone();
                                    let mut counters = counters_clone2.lock().unwrap();
                                    counters.1 += 1;
                                    drop(counters);
                                    Either::A(channel.basic_ack(delivery_tag, false).inspect(move |_| {
                                        let mut counters = counters_clone.lock().unwrap();
                                        counters.2 += 1;
                                        drop(counters);
                                    }))
                                }
                                Err(e) => {
                                    let counters_clone = counters_clone2.clone();
                                    let mut counters = counters_clone2.lock().unwrap();
                                    counters.3 += 1;
                                    drop(counters);
                                    log_error(&e);
                                    let when = Instant::now() + Duration::from_millis(DELAY_BEFORE_NACK);
                                    let f = Delay::new(when).then(move |_| {
                                        channel.basic_nack(delivery_tag, false, true).inspect(move |_| {
                                            let mut counters = counters_clone.lock().unwrap();
                                            counters.4 += 1;
                                            drop(counters);
                                        })
                                    });
                                    tokio::spawn(f.map_err(|e| {
                                        error!("Error sending nack: {}", e);
                                    }));
                                    Either::B(future::ok(()))
                                }
                            })
                        }).map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                }).map_err(|e| {
                    log_error(&e);
                });
            let _ = core
                .run(Timeout::new(subscription, resubscribe_duration).then(move |_| {
                    let counters = counters_clone.lock().unwrap();
                    info!(
                        "Total messages: {}, tried to ack: {}, acked: {}, tried to nack: {}, nacked: {}",
                        counters.0, counters.1, counters.2, counters.3, counters.4
                    );
                    drop(counters);
                    let fs: Vec<_> = consumers_to_close_clone
                        .lock()
                        .unwrap()
                        .iter_mut()
                        .map(|(channel, consumer_tag)| {
                            let mut channel = channel.clone();
                            let consumer_tag = consumer_tag.clone();
                            trace!("Canceling {} with channel `{}`", consumer_tag, channel.id);
                            channel
                                .cancel_consumer(consumer_tag.to_string())
                                .and_then(move |_| channel.close(0, "Cancelled on consumer resubscribe"))
                        }).collect();
                    future::join_all(fs)
                })).map(|_| ())
                .map_err(|e: io::Error| {
                    error!("Error closing consumer {}", e);
                });
        }
    });

    api::start_server(config, publisher.clone());
}

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
