use futures::future;
use futures_cpupool::CpuPool;
use lapin_futures::channel::BasicConsumeOptions;
use lapin_futures::channel::Channel;
use lapin_futures::channel::QueueDeclareOptions;
use lapin_futures::consumer::Consumer;
use lapin_futures::types::FieldTable;
use r2d2::PooledConnection;
use tokio::net::tcp::TcpStream;

use super::error::*;
use super::r2d2::RabbitConnectionManager;
use super::r2d2::RabbitPool;
use models::*;
use prelude::*;

#[derive(Clone)]
pub struct TransactionConsumerImpl {
    rabbit_pool: RabbitPool,
    thread_pool: CpuPool,
    workspace_id: WorkspaceId,
}

impl TransactionConsumerImpl {
    pub fn new(rabbit_pool: RabbitPool, thread_pool: CpuPool, workspace_id: WorkspaceId) -> Self {
        Self {
            rabbit_pool,
            thread_pool,
            workspace_id,
        }
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
                ).map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                .and_then(move |queue| {
                    channel_clone
                        .basic_consume(&queue, "", BasicConsumeOptions::default(), FieldTable::new())
                        .map(move |consumer| (consumer, channel_clone))
                        .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
                })
        })
    }

    fn get_channel(&self) -> impl Future<Item = PooledConnection<RabbitConnectionManager>, Error = Error> {
        // unresolved at the moment - ideally we want to call get on other thread, since it's blocking
        // on the other hand doing so we escape from the thread that has tokio core reference and
        // therefore cannot do spawns
        // let rabbit_pool = self.rabbit_pool.clone();
        // self.thread_pool
        //     .spawn_fn(move || rabbit_pool.get().map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal)))
        self.rabbit_pool
            .get()
            .map_err(ectx!(ErrorSource::Lapin, ErrorKind::Internal))
            .into_future()
    }
}
