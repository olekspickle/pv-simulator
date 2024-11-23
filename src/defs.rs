use anyhow::anyhow;
use deadpool_lapin::{Pool, Runtime};
use rand::Rng;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

const AMQP_ADDR: &str = "amqp://rmq:rmq@127.0.0.1:5672/%2f";
const METER_QUEUE_NAME: &str = "meter_queue";
const PV_QUEUE_NAME: &str = "pv_queue";

#[derive(Clone, Debug)]
pub struct Meter {
    min: u32,
    max: u32,
}

impl Meter {
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    /// Generates random value in Watts with the meter min/max values
    pub fn consume(&self) -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.min..self.max)
    }
}

/// Struct that will manage the simulation loop
#[derive(Clone, Debug)]
pub struct Simulation {
    meter: Meter,
    values: Vec<u32>,
}

impl Simulation {
    pub fn new(meter: Meter) -> Self {
        Self {
            meter,
            values: vec![],
        }
    }

    /// Returns a reference for a meter
    fn meter(&self) -> &Meter {
        &self.meter
    }

    /// Approximate load data function for a 24h cycle
    ///
    /// Returns: power value in kW
    pub fn load(h: f32) -> f32 {
        match h {
            h if h < 8.75 && h > 19.25 => 3.5 - (h - 14.0).powi(2) / 9.0,
            _ => 1.5 - (h - 14.0).powi(2) / 26.0,
        }
    }

    /// Starts the simulation loop with given time frame
    pub async fn start(&self, time_range: std::ops::Range<u8>) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut buf: &[u8] = &[];
        let pool = Self::pool()?;

        for n in time_range {
            let h = n as f32;
            let meter_value = self.meter().consume();
            println!("{h}:{}", meter_value);
            let con = pool.get().await;
            

            interval.tick().await;
        }

        csv::ReaderBuilder::new().from_reader(buf);

        Ok(())
    }

    /// Returns result of creating a new pool for amqp connections
    fn pool() -> Result<Pool> {
        // Optional: you can set amqp address as env var
        let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| AMQP_ADDR.into());
        let pool_cfg = deadpool_lapin::Config {
            url: Some(addr),
            ..Default::default()
        };
        pool_cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| anyhow!("failed to create pool:{}", e))
    }
}
