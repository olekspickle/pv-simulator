//! ### Overview
//! 
//! The idea is to simulate **PV generator** values interaction of Solar panels
//! and building consumption measured by a Meter.
//!
//! Consuption is measured by **meter** and then added to RabbitMQ instance.
//! From the rmq instance **PV generator** then takes values of a meter
//! and outputs summarized values.
//!
//! The simulation should record each second with the meter values 0-9000 Watts.
//!
//! The end result of a simulation is a CSV file.
//! 
//! ### How to run
//!     1. Run rabbitmq:
//!     ```bash
//!         docker compose up
//!     ```
//! 
//!     2. Run the app:
//!     ```bash
//!         RUST_LOG=info cargo run
//!     ```
//! 
//!
pub(crate) mod meter;
pub(crate) mod pool;
pub(crate) mod sim;

use meter::Meter;
use sim::Simulation;

const AMQP_ADDR: &str = "amqp://user:pass@127.0.0.1:5672/%2f";
const METER_QUEUE: &str = "meter_queue";
const SECS: u32 = 86400;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    Simulation::start(Meter::new(0, 9000), 0..SECS).await?;

    Ok(())
}
