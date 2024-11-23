use anyhow::anyhow;
use deadpool_lapin::Runtime;
use lapin::{options::*, BasicProperties, Channel, Queue};
use rand::Rng;
use std::time::Duration;

use crate::SECS;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

const AMQP_ADDR: &str = "amqp://rmq:rmq@127.0.0.1:5672/%2f";
const METER_QUEUE: &str = "meter_queue";
const PV_QUEUE_NAME: &str = "pv_queue";

#[derive(Clone, Debug)]
pub struct Meter {
    min: u32,
    max: u32,
}

impl Meter {
    /// Creates a new [`Meter`] with upper and lower value bounds in Watts
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    /// Generates random value in Watts with the meter min/max values
    pub fn consume(&self) -> f32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.min..self.max) as f32
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
    pub fn generated(h: f32) -> f32 {
        match h {
            h if h < 8.75 && h > 19.25 => 3.5 - (h - 14.0).powi(2) / 9.0,
            _ => 1.5 - (h - 14.0).powi(2) / 26.0,
        }
    }

    /// Starts the simulation loop with given time frame
    pub async fn start(&self, time_range: std::ops::Range<u32>) -> Result<()> {
        let mut buf: Vec<u8> = b"timestamp,meter_value,pv_value,resulting\n".to_vec();
        let pool = Pool::new()?;

        // In order to run simulation in order of a few minutes
        // Meter values will be produced approximately 360 times per second (60Hz)
        // This way the whole 24h cycle will run in 240s(4m) timespan
        let mut interval = tokio::time::interval(Duration::from_micros(250));
        let _ = pool.declare_queue();

        // launch meter generation in one thread
        // tokio::spawn(async {
        //     for n in time_range {
        //         let meter_value = self.meter().consume();
        //         let payload = meter_value.to_ne_bytes();
        //         let channel = pool.channel().await?;

        //         // TODO: we might want to do something with publish confirmation
        //         let _ = channel
        //             .basic_publish(
        //                 "",
        //                 METER_QUEUE,
        //                 BasicPublishOptions::default(),
        //                 &payload,
        //                 BasicProperties::default(),
        //             )
        //             .await;
        //     }

        //     Ok(())
        // })
        // .await?;

        // tokio::spawn(async{
        //     let h = (n / SECS) as f32;
        //     let ts = chrono::Utc::now();
        //     let pv_value = Self::generated(h);
        //     let resulting = pv_value - meter_value / 1000.0;

        //     let new = format!("{ts},{meter_value},{pv_value},{resulting}\n");
        //     buf = [buf, new.as_bytes().to_vec()].concat();

        //     interval.tick().await;
        // });

        let reader = csv::ReaderBuilder::new().from_reader(&*buf);

        Ok(())
    }
}

/// Wrapper to encapsulate pool logic
#[derive(Debug, Clone)]
struct Pool(deadpool_lapin::Pool);

impl Pool {
    /// Returns result of a new pool creation for amqp connections
    fn new() -> Result<Pool> {
        // Optional: you can set amqp address as env var for amqp address
        let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| AMQP_ADDR.into());
        let pool_cfg = deadpool_lapin::Config {
            url: Some(addr),
            ..Default::default()
        };
        pool_cfg
            .create_pool(Some(Runtime::Tokio1))
            .map(|inner| Self(inner))
            .map_err(|e| anyhow!("Failed to create pool:{}", e))
    }

    /// Creates a channel for current connection
    pub async fn channel(&self) -> Result<Channel> {
        let con = self.0.get().await?;
        con.create_channel()
            .await
            .map_err(|e| anyhow!("Failed to create channel:{e}"))
    }

    /// Declares a queue for meter
    pub async fn declare_queue(&self) -> Result<Queue> {
        let channel = self.channel().await?;
        channel
            .queue_declare(METER_QUEUE, Default::default(), Default::default())
            .await
            .map_err(|e| anyhow!("Failed to declare queue:{e}"))
    }
}
