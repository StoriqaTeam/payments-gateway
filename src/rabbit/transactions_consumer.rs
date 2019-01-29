use lapin_futures::channel::{BasicConsumeOptions, Channel, QueueDeclareOptions};
use lapin_futures::consumer::Consumer;
use lapin_futures::types::FieldTable;
use tokio::net::tcp::TcpStream;

use super::error::*;
use super::r2d2::RabbitConnectionManager;
use models::*;
use prelude::*;

#[derive(Clone)]
pub struct TransactionConsumerImpl {
    rabbit_pool: RabbitConnectionManager,
    workspace_id: WorkspaceId,
}

impl TransactionConsumerImpl {
    pub fn new(rabbit_pool: RabbitConnectionManager, workspace_id: WorkspaceId) -> Self {
        Self { rabbit_pool, workspace_id }
    }

    pub fn subscribe(&self) -> impl Future<Item = (Consumer<TcpStream>, Channel<TcpStream>), Error = Error> {
        let queue_name = format!("transactions_{}", self.workspace_id);
        self.get_channel().and_then(move |channel| {
            let channel_clone = channel.clone();
            channel
                .queue_declare(
                    &queue_name,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    Default::default(),
                )
                .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                .and_then(move |queue| {
                    channel_clone
                        .basic_consume(&queue, "", BasicConsumeOptions::default(), FieldTable::new())
                        .map(move |consumer| (consumer, channel_clone))
                        .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                })
        })
    }

    fn get_channel(&self) -> impl Future<Item = Channel<TcpStream>, Error = Error> {
        self.rabbit_pool
            .get_channel()
            .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
            .into_future()
    }
}
