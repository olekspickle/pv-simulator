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

/// Interval in microsecs to wait between send and receive
/// current number will define how much time the whole simulation will take
const DEFAULT_WAIT: Duration = Duration::from_micros(250);

/// Struct that will manage the simulation loop
#[derive(Clone, Debug)]
pub struct Simulation {
    pool: Pool,
}

impl Simulation {
    pub fn new() -> Result<Self> {
        let pool = Pool::new()?;
        Ok(Self { pool })
    }

    /// Approximate load data function for a 24h cycle
    ///
    /// Returns: power value in kW
    /// Created with Desmos: https://www.desmos.com/calculator/03gcro3e4d
    pub fn generate(h: f32) -> f32 {
        let p = match h {
            h if h < 8.0 && h > 20.0 => 1.5 - (h - 14.0).powi(2) / 32.0,
            _ => 3.2 - (h - 14.0).powi(2) / 11.0,
        };
        match p {
            p if p < 0.0 => 0.0,
            p => p,
        }
    }

    /// Starts the simulation loop with given time frame
    ///
    /// In order to run simulation in order of a few minutes
    /// Meter values will be produced approximately 612 times per second (~160Hz)
    /// This way the whole 24h cycle will run in 140s(2m20s) timespan
    pub async fn start(&self, meter: Meter, time_range: std::ops::Range<u32>) -> Result<()> {
        Self::reset_previous_results()?;

        let queue = self.pool.declare_queue().await?;
        info!("Declared the '{:?}' queue", queue);

        // Lifetimes in threads: we need it just to manage channel in a thread scope
        let pub_channel = self.pool.channel().await?;
        // 1. Launch meter generation process in one task
        info!("Launching meter values generation task...");
        tokio::spawn(async move {
            for n in time_range {
                let meter_record = MeterRecord::new(n, meter.consume());
                let payload = meter_record.to_string().as_bytes().to_vec();
                debug!(
                    "{n}-Meter payload:{:?}",
                    String::from_utf8(payload.clone()).unwrap()
                );

                pub_channel
                    .basic_publish(
                        "", // exchange
                        METER_QUEUE,
                        BasicPublishOptions::default(),
                        &payload,
                        BasicProperties::default(),
                    )
                    .await?;

                Self::tick().await;
            }

            Ok::<(), anyhow::Error>(())
        });

        // 2. Launch pv receiving in another task
        info!("Launching PV subscriber...");
        let mut sub = self.consumer().await?;
        loop {
            // Without this workaround the process hangs at the end on the amqp async stream loop.
            // This is what I came up with to break it gracefully, but async is still hard :(
            // So it looks terrible but justifiable in this kind of project.
            let sleep = tokio::time::sleep(Duration::from_secs(2));
            tokio::pin!(sleep);
            let delivery = tokio::select! {
                success = sub.try_next() => Ok(success?),
                _ = &mut sleep => Err(anyhow::anyhow!("No messages"))
            };

            if delivery.is_err() {
                break;
            }

            let delivery = delivery?.expect("No messages");
            let record: MeterRecord = delivery.data.clone().into();

            let meter_value = record.value();
            let pv_value = Self::generate(record.hours());
            let resulting = meter_value / 1000.0 - pv_value;
            debug!("{meter_value}/{} - {pv_value}={resulting}", 1000.0);

            let dt = record.datetime().to_string();
            let row = format!("{dt},{meter_value},{pv_value},{resulting}\n");
            Self::write_row(&mut row.as_bytes().to_vec())?;

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("ACK failed");
        }

        info!("Done");

        Ok(())
    }

    /// Truncate file and add columns to it
    fn reset_previous_results() -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("results.csv")
            .unwrap();

        file.write_all(b"timestamp,meter_value,pv_value,resulting\n")?;

        Ok(())
    }

    /// Append buffer to a file
    fn write_row(buf: &mut Vec<u8>) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("results.csv")
            .unwrap();
        file.write_all(buf)?;

        Ok(())
    }

    /// Create a consumer with from the current pool's connection
    async fn consumer(&self) -> Result<lapin::Consumer> {
        let sub_channel = self
            .pool
            .channel()
            .await
            .expect("failed to create a sub connection");
        let sub = sub_channel
            .basic_consume(
                METER_QUEUE,
                "test_consume",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("failed to create a subscription");

        Ok(sub)
    }

    async fn _check_message_count(&self) -> Result<u32> {
        let ch = self.pool.channel().await?;
        let queue = ch
            .queue_declare(
                METER_QUEUE,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(queue.message_count())
    }

    /// Wait some time
    async fn tick() {
        let mut interval = tokio::time::interval(DEFAULT_WAIT);
        interval.tick().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn generate_pv() {
        // 8:30
        let h = Simulation::generate(8.5);
        expect!["0.45000005"].assert_eq(&h.to_string());

        // 10:30
        let h = Simulation::generate(10.5);
        expect!["2.0863638"].assert_eq(&h.to_string());

        // 12:30
        let h = Simulation::generate(12.5);
        expect!["2.9954545"].assert_eq(&h.to_string());

        // 14:30
        let h = Simulation::generate(14.5);
        expect!["3.1772728"].assert_eq(&h.to_string());

        // 16:42
        let h = Simulation::generate(16.7);
        expect!["2.5372725"].assert_eq(&h.to_string());

        // 17:42
        let h = Simulation::generate(17.7);
        expect!["1.9554541"].assert_eq(&h.to_string());

        // 18:42
        let h = Simulation::generate(18.7);
        expect!["1.1918175"].assert_eq(&h.to_string());

        // 20:42
        let h = Simulation::generate(20.1);
        expect!["0"].assert_eq(&h.to_string());
    }
}
