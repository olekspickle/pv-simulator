use crate::{
    meter::{Meter, MeterRecord},
    pool::Pool,
    METER_QUEUE,
};
use anyhow::Result;
use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, BasicProperties};
use log::{debug, info};
use std::{fs, io::Write, time::Duration};

/// Struct that will manage the simulation loop
#[derive(Clone, Debug)]
pub struct Simulation;

impl Simulation {
    /// Approximate load data function for a 24h cycle
    ///
    /// Returns: power value in kW
    /// Created with Desmos: https://www.desmos.com/calculator/s2lfq2nftl
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
        info!("Declared the '{:?}' queue", queue);

        // Lifetimes in threads: we need it just for creating a channel in a thread scope
        let pool_1 = pool.clone();
        // 1. Launch meter generation process in one task
        info!("Launching generation task...");
        tokio::spawn(async move {
            // let mut interval = tokio::time::interval(Duration::from_micros(250));
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            for n in time_range {
                let meter_value: MeterRecord = meter.consume().into();
                let payload = meter_value.to_string().as_bytes().to_vec();
                debug!(
                    "{n}-Message payload:{:?}",
                    String::from_utf8(payload.clone()).unwrap()
                );

                let channel = pool_1.channel().await?;
                channel
                    .basic_publish(
                        "", // exchange
                        METER_QUEUE,
                        BasicPublishOptions::default(),
                        &payload,
                        BasicProperties::default(),
                    )
                    .await?;

                debug!("{n}-Message published: {}", meter_value.value());
                interval.tick().await;
            }

            Ok::<(), anyhow::Error>(())
        });

        info!("Launching PV subscriber...");
        // 2. Launch pv receiving in another task
        let channel = pool
            .channel()
            .await
            .expect("failed to create a sub connection");
        let mut sub = channel
            .basic_consume(
                METER_QUEUE,
                "test_consume",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("failed to create a subscription");

        Self::write_row(&mut b"timestamp,meter_value,pv_value,resulting\n".to_vec())?;

        while let Some(delivery) = sub.next().await {
            let delivery = delivery.expect("error in consumer");
            let record: MeterRecord = delivery.data.clone().into();

            let dt = record.datetime().to_string();
            let h = record.hours();
            let meter_value = record.value();
            let pv_value = Self::generated(h);
            let resulting = pv_value - meter_value / 1000.0;

            let row = format!("{dt},{meter_value},{pv_value},{resulting}\n");
            Self::write_row(&mut row.as_bytes().to_vec())?;
            debug!("Row appended to the file: {}", &row);

            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }

        Ok(())
    }

    /// Write final buffer in a file
    fn write_row(buf: &mut Vec<u8>) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("results.csv")
            .unwrap();
        file.write_all(buf).expect("Could not write to a file");
        // let mut rdr = csv::ReaderBuilder::new().from_reader(&**buf);
        // let mut wtr = csv::WriterBuilder::new().from_path("results.csv")?;
        // let mut record = csv::ByteRecord::new();
        // while rdr.read_byte_record(&mut record)? {
        //     wtr.write_byte_record(&record)?;
        // }
        // wtr.flush()?;

        Ok(())
    }
}
