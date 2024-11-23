use anyhow::anyhow;
use deadpool_lapin::Runtime;
use lapin::{
    options::QueueDeclareOptions, types::FieldTable, Channel, ConnectionProperties, Queue,
};

/// Wrapper to encapsulate some connection logic around the pool
#[derive(Debug, Clone)]
pub(crate) struct Pool(deadpool_lapin::Pool);

impl Pool {
    /// Returns result of a new pool creation for amqp connections
    ///
    /// Optional: you can set amqp address as env var for amqp address
    pub fn new() -> anyhow::Result<Pool> {
        let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| crate::AMQP_ADDR.into());
        let pool_cfg = deadpool_lapin::Config {
            url: Some(addr),
            connection_properties: ConnectionProperties::default()
                .with_connection_name("meter_connection_pool".into()),
            ..Default::default()
        };
        pool_cfg
            .create_pool(Some(Runtime::Tokio1))
            .map(Self)
            .map_err(|e| anyhow!("Failed to create pool:{}", e))
    }

    /// Creates a channel for current connection
    pub async fn channel(&self) -> anyhow::Result<Channel> {
        let con = self.0.get().await?;
        con.create_channel()
            .await
            .map_err(|e| anyhow!("Failed to create channel:{e}"))
    }

    /// Declares a queue for meter
    pub async fn declare_queue(&self) -> anyhow::Result<Queue> {
        let channel: Channel = self.channel().await?;
        channel
            .queue_declare(
                crate::METER_QUEUE,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| anyhow!("Failed to declare queue:{e}"))
    }
}
