use borsh::{BorshDeserialize, BorshSerialize};
use chrono::{DateTime, FixedOffset, Local};
use rand::Rng;

/// Meter payload message with time and amplitude
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub(crate) struct MeterValue {
    /// Timestamp of a consume power record
    time: String,
    /// Value in Watts
    value: String,
}

impl MeterValue {
    pub fn datetime(&self) -> DateTime<FixedOffset> {
        log::debug!("trying to parse datetime: {}", self.value);
        DateTime::parse_from_rfc2822(&self.value).expect("Tried to parse Datetime from string")
    }
}

impl From<f32> for MeterValue {
    fn from(v: f32) -> Self {
        let time = chrono::Local::now().to_string();
        Self {
            time,
            value: v.to_string(),
        }
    }
}

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
