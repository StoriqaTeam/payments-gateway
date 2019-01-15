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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use failure::Error as FailureError;
use failure::Fail;
use futures::future::{self, Either};
use futures_cpupool::CpuPool;
use lapin_futures::channel::Channel;
use tokio::net::tcp::TcpStream;
use tokio::prelude::*;
use tokio::timer::{Delay, Timeout};

use self::models::*;
use config::Config;
use rabbit::{ErrorKind, ErrorSource};
use rabbit::{R2D2ErrorHandler, RabbitConnectionManager, TransactionConsumerImpl, TransactionPublisherImpl};
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

    let mut core = tokio_core::reactor::Core::new().unwrap();
    debug!("Started creating rabbit connection pool");

    let rabbit_thread_pool = futures_cpupool::CpuPool::new(config_clone.rabbit.thread_pool_size);
    let rabbit_connection_manager = core
        .run(RabbitConnectionManager::create(&config_clone))
        .map_err(|e| {
            log_error(&e);
        })
        .unwrap();
    let rabbit_connection_pool = r2d2::Pool::builder()
        .max_size(config_clone.rabbit.connection_pool_size as u32)
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
        })
        .unwrap();
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
            let counters = Rc::new(RefCell::new((0usize, 0usize, 0usize, 0usize, 0usize)));
            let counters_clone = counters.clone();
            let consumers_to_close: Rc<RefCell<Vec<(Channel<TcpStream>, String)>>> = Rc::new(RefCell::new(Vec::new()));
            let consumers_to_close_clone = consumers_to_close.clone();
            let last_delivery_tag: Rc<RefCell<HashMap<String, u64>>> = Rc::new(RefCell::new(HashMap::new()));
            let last_delivery_tag_clone = last_delivery_tag.clone();
            let resubscribe_duration = Duration::from_secs(config_clone.rabbit.restart_subscription_secs as u64);
            let fetcher_clone = fetcher.clone();
            let subscription = consumer
                .subscribe()
                .and_then(move |(stream, channel)| {
                    let counters_clone = counters.clone();
                    let fetcher_clone = fetcher_clone.clone();
                    let consumers_to_close = consumers_to_close.clone();
                    let last_delivery_tag_clone = last_delivery_tag.clone();
                    let mut consumers_to_close_lock = consumers_to_close.borrow_mut();
                    let counsumer_tag = stream.consumer_tag.clone();
                    consumers_to_close_lock.push((channel.clone(), stream.consumer_tag.clone()));
                    stream
                        .for_each(move |message| {
                            trace!("got message: {}", MessageDelivery::new(message.clone()));
                            let delivery_tag = message.delivery_tag;
                            let mut counters = counters_clone.borrow_mut();
                            counters.0 += 1;
                            let counters_clone2 = counters_clone.clone();
                            let channel = channel.clone();
                            let last_delivery_tag_clone2 = last_delivery_tag_clone.clone();
                            let mut last_delivery_tag_clone = last_delivery_tag_clone.borrow_mut();
                            last_delivery_tag_clone.insert(counsumer_tag.clone(), delivery_tag);
                            let counsumer_tag = counsumer_tag.clone();
                            fetcher_clone.handle_message(message.data).then(move |res| {
                                let mut last_delivery_tag_clone = last_delivery_tag_clone2.borrow_mut();
                                last_delivery_tag_clone.remove(&counsumer_tag);
                                match res {
                                    Ok(_) => {
                                        let mut counters_clone = counters_clone2.clone();
                                        let mut counters = counters_clone.borrow_mut();
                                        counters.1 += 1;
                                        Either::A(channel.basic_ack(delivery_tag, false).inspect(move |_| {
                                            let counters_clone = counters_clone2.clone();
                                            let mut counters = counters_clone.borrow_mut();
                                            counters.2 += 1;
                                        }))
                                    }
                                    Err(e) => {
                                        let mut counters_clone = counters_clone2.clone();
                                        let mut counters = *counters_clone.borrow_mut();
                                        counters.3 += 1;
                                        log_error(&e);
                                        let when = Instant::now() + Duration::from_millis(DELAY_BEFORE_NACK);
                                        let f = Delay::new(when).then(move |_| {
                                            channel.basic_nack(delivery_tag, false, true).inspect(move |_| {
                                                counters.4 += 1;
                                            })
                                        });
                                        tokio::spawn(f.map_err(|e| {
                                            error!("Error sending nack: {}", e);
                                        }));
                                        Either::B(future::ok(()))
                                    }
                                }
                            })
                        })
                        .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                })
                .map_err(|e| {
                    log_error(&e);
                });
            let _ = core.run(
                Timeout::new(subscription, resubscribe_duration)
                    .then(move |_| {
                        let counters = counters_clone.borrow();
                        info!(
                            "Total messages: {}, tried to ack: {}, acked: {}, tried to nack: {}, nacked: {}",
                            counters.0, counters.1, counters.2, counters.3, counters.4
                        );
                        let mut consumers_to_close_lock = consumers_to_close_clone.borrow_mut();
                        let last_delivery_tag_clone2 = last_delivery_tag_clone.clone();
                        let last_delivery_tag_lock = last_delivery_tag_clone2.borrow_mut();
                        let fs: Vec<_> = consumers_to_close_lock
                            .iter_mut()
                            .map(move |(channel, consumer_tag)| {
                                let mut channel = channel.clone();
                                let channel_clone = channel.clone();
                                let consumer_tag = consumer_tag.clone();
                                let last_delivery_tag = last_delivery_tag_lock.get(&consumer_tag.to_string()).cloned();
                                trace!("Canceling {} with channel `{}`", consumer_tag, channel.id);
                                if let Some(last_delivery_tag) = last_delivery_tag {
                                    Either::A(channel.basic_nack(last_delivery_tag, true, true))
                                } else {
                                    Either::B(future::ok(()))
                                }
                                .map_err(From::from)
                                .and_then(move |_| channel.cancel_consumer(consumer_tag.to_string()).map_err(From::from))
                                .and_then(move |_| {
                                    let mut transport = channel_clone.transport.lock().unwrap();
                                    transport.conn.basic_recover(channel_clone.id, true).map_err(From::from)
                                })
                            })
                            .collect();

                        future::join_all(fs)
                    })
                    .map(|_| ())
                    .map_err(|e: FailureError| {
                        error!("Error closing consumer {}", e);
                    })
                    .then(move |_| {
                        let when = Instant::now() + Duration::from_millis(DELAY_BEFORE_RECONNECT);
                        Delay::new(when)
                    }),
            );
        }
    });

    api::start_server(config, publisher.clone());
}

fn get_config() -> Config {
    config::Config::new().unwrap_or_else(|e| panic!("Error parsing config: {}", e))
}
