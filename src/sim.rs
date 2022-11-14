use crate::{
    meter::{Meter, MeterValue},
    pool::Pool,
};
use chrono::{DateTime, FixedOffset, Local};

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, BasicProperties};
use std::time::Duration;

/// Struct that will manage the simulation loop
#[derive(Clone, Debug)]
pub struct Simulation;

impl Simulation {
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
    ///
    /// In order to run simulation in order of a few minutes
    /// Meter values will be produced approximately 360 times per second (60Hz)
    /// This way the whole 24h cycle will run in 240s(4m) timespan
    pub async fn start(meter: Meter, time_range: std::ops::Range<u32>) -> Result<()> {
        let pool = Pool::new()?;

        let queue = pool.declare_queue().await?;
        println!("Declared the '{:?}' queue", queue);

        // Lifetimes: we need it just for creating a channel in a thread scope
        let pool_1 = pool.clone();
        // 1. Launch meter generation in one task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_micros(250));
            for _ in time_range {
                let meter_value: MeterValue = meter.consume().into();
                let payload = meter_value.try_to_vec()?;

                let channel = pool_1.channel().await?;
                let p = channel
                    .basic_publish(
                        "", // exchange
                        crate::METER_QUEUE,
                        BasicPublishOptions::default(),
                        &payload,
                        BasicProperties::default(),
                    )
                    .await?;

                println!("Meter record published to the queue:{:?}", p);
                interval.tick().await;
            }

            Ok::<(), anyhow::Error>(())
        });

        // 2. Launch pv receiving in another task
        let channel = pool
            .channel()
            .await
            .expect("failed to create a sub connection");
        let mut sub = channel
            .basic_consume(
                "",
                crate::METER_QUEUE,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("failed to create a subscription");

        let mut buf: Vec<u8> = b"timestamp,meter_value,pv_value,resulting\n".to_vec();
        while let Some(delivery) = sub.next().await {
            let delivery = delivery.expect("error in consumer");
            let record = MeterValue::try_from_slice(&delivery.data).unwrap();
            let dt: DateTime<_> = record.datetime();
            println!("time:{}", dt);
            buf.append(&mut dt.to_string().as_bytes().to_vec());
            // let today = time
            // let h = (n / SECS) as f32;
            // let ts = Local::now();
            // let pv_value = Self::generated(h);
            // let resulting = pv_value - record.value / 1000.0;

            // let new = format!("{ts},{meter_value},{pv_value},{resulting}\n");
            // buf = [buf, new.as_bytes().to_vec()].concat();

            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }

        Self::save_results(&mut buf)?;
        Ok(())
    }

    fn save_results(buf: &mut Vec<u8>) -> Result<()> {
        let mut rdr = csv::ReaderBuilder::new().from_reader(&**buf);
        let mut wtr = csv::WriterBuilder::new().from_path("results.csv")?;
        let mut record = csv::ByteRecord::new();
        while rdr.read_byte_record(&mut record)? {
            wtr.write_byte_record(&record)?;
        }
        wtr.flush()?;

        Ok(())
    }
}
