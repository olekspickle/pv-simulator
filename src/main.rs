use deadpool_lapin::lapin::{options::BasicPublishOptions, BasicProperties};
use deadpool_lapin::{Config, Manager, Pool, Runtime};
use lapin::ConnectionProperties;

const DEFAULT_QUEUE: &str = "amqp://rmq:rmq@127.0.0.1:5672/%2f";
const METER_QUEUE_NAME: &str = "meter_queue";
const PV_QUEUE_NAME: &str = "pv_queue";

/// Load data for a 24h cycle
/// TODO: make a function where:
/// VAL = y - (x - 14)^2 / xd
/// before 8:45 & after 19:15 ->  y = 1.5; xd = 26
/// and in between            ->  y = 3.5; xd = 9
/// 
const LOAD_BIASES: &[(u8, f32)] = &[
    (0, 0.0),
    (6, 0.0),
    (8, 0.25),
    (10, 1.75),
    (11, 2.5),
    (12, 2.75),
    (14, 3.5),
    (16, 3.0),
    (17, 2.5),
    (18, 1.75),
    (20, 0.15),
    (21, 0.0),
    (24, 0.0),
];

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let pool_cfg = Config {
        url: Some(DEFAULT_QUEUE.into()),
        ..Default::default()
    };
    let pool = pool_cfg.create_pool(Some(Runtime::Tokio1))?;
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| DEFAULT_QUEUE.into());

    Ok(())
}
