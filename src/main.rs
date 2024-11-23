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
mod defs;
mod error;

use defs::{Meter, Simulation};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let meter = Meter::new(0, 9000);
    let sim = Simulation::new(meter);

    sim.start(0..24).await?;

    Ok(())
}
