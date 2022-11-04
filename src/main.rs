//! The idea is to simulate PV values interaction of Solar panels and consumption.
//!
//! Consuption is measured by meter and then added to RabbitMQ instance.
//! From the rmq instance PV generator then takes values of a meter
//! and outputs summarized values.
//!
//! The simulation should record each second with the meter values 0-9000 Watts.
//!
//! The end result of a simulation is a CSV file.
//!

use deadpool_lapin::lapin::{options::BasicPublishOptions, BasicProperties};
use deadpool_lapin::{Config, Manager, Pool, Runtime};
use lapin::ConnectionProperties;

const DEFAULT_QUEUE: &str = "amqp://rmq:rmq@127.0.0.1:5672/%2f";
const METER_QUEUE_NAME: &str = "meter_queue";
const PV_QUEUE_NAME: &str = "pv_queue";

/// Approximate load data function for a 24h cycle
fn load(h: f32) -> f32 {
    match h {
        h if h < 8.75 && h > 19.25 => 3.5 - (h - 14.0).powi(2) / 9.0,
        _ => 1.5 - (h - 14.0).powi(2) / 26.0,
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let pool_cfg = Config {
        url: Some(DEFAULT_QUEUE.into()),
        ..Default::default()
    };
    let pool = pool_cfg.create_pool(Some(Runtime::Tokio1))?;
    // Optionally uou can set amqp address as env var
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| DEFAULT_QUEUE.into());

    Ok(())
}
