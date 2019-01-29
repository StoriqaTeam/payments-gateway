use std::sync::Arc;

use futures::future;
use lapin_futures::channel::{Channel, ExchangeDeclareOptions, QueueDeclareOptions};
use lapin_futures::error::Error as LapinError;
use serde_json;
use tokio::net::tcp::TcpStream;

use super::error::*;
use super::r2d2::RabbitConnectionManager;
use models::*;
use prelude::*;

pub trait TransactionPublisher: Send + Sync + 'static {
    fn push(&self, push: PushNotifications) -> Box<Future<Item = (), Error = Error> + Send>;
    fn callback(&self, callback: Callback) -> Box<Future<Item = (), Error = Error> + Send>;
    fn send_email(&self, email: Email) -> Box<Future<Item = (), Error = Error> + Send>;
}
#[derive(Clone)]
pub struct TransactionPublisherImpl {
    channel: Arc<Channel<TcpStream>>,
}

impl TransactionPublisherImpl {
    pub fn new(rabbit_pool: RabbitConnectionManager) -> Self {
        let channel = Arc::new(rabbit_pool.get_channel().expect("Can not get channel from pool"));
        Self { channel }
    }

    pub fn init(&mut self) -> impl Future<Item = (), Error = Error> {
        let channel = self.channel.clone();

        let f1: Box<Future<Item = (), Error = LapinError>> = Box::new(channel.exchange_declare(
            "notifications",
            "direct",
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            Default::default(),
        ));
        let f2: Box<Future<Item = (), Error = LapinError>> = Box::new(
            channel
                .queue_declare(
                    "pushes",
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    Default::default(),
                )
                .map(|_| ()),
        );
        let f3: Box<Future<Item = (), Error = LapinError>> = Box::new(
            channel
                .queue_declare(
                    "callbacks",
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    Default::default(),
                )
                .map(|_| ()),
        );
        let f4: Box<Future<Item = (), Error = LapinError>> = Box::new(
            channel
                .queue_declare(
                    "emails",
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    Default::default(),
                )
                .map(|_| ()),
        );
        let f5: Box<Future<Item = (), Error = LapinError>> =
            Box::new(channel.queue_bind("pushes", "notifications", "pushes", Default::default(), Default::default()));
        let f6: Box<Future<Item = (), Error = LapinError>> =
            Box::new(channel.queue_bind("callbacks", "notifications", "callbacks", Default::default(), Default::default()));
        let f7: Box<Future<Item = (), Error = LapinError>> =
            Box::new(channel.queue_bind("emails", "notifications", "emails", Default::default(), Default::default()));
        future::join_all(vec![f1, f2, f3, f4, f5, f6, f7])
            .map(|_| ())
            .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
    }
}

impl TransactionPublisher for TransactionPublisherImpl {
    fn push(&self, push: PushNotifications) -> Box<Future<Item = (), Error = Error> + Send> {
        let channel = self.channel.clone();
        let payload = serde_json::to_string(&push).unwrap().into_bytes();
        Box::new(
            channel
                .clone()
                .basic_publish("notifications", "pushes", payload, Default::default(), Default::default())
                .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                .map(|_| ()),
        )
    }
    fn callback(&self, callback: Callback) -> Box<Future<Item = (), Error = Error> + Send> {
        let channel = self.channel.clone();
        let payload = serde_json::to_string(&callback).unwrap().into_bytes();
        Box::new(
            channel
                .clone()
                .basic_publish("notifications", "callbacks", payload, Default::default(), Default::default())
                .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                .map(|_| ()),
        )
    }
    fn send_email(&self, email: Email) -> Box<Future<Item = (), Error = Error> + Send> {
        let channel = self.channel.clone();
        let payload = serde_json::to_string(&email).unwrap().into_bytes();
        Box::new(
            channel
                .clone()
                .basic_publish("notifications", "emails", payload, Default::default(), Default::default())
                .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                .map(|_| ()),
        )
    }
}
