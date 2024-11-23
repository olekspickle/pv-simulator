use chrono::{DateTime, Local, Timelike};
use rand::Rng;

/// Meter payload message with time and amplitude
#[derive(PartialEq, Debug)]
pub(crate) struct MeterRecord {
    /// Second of simulation
    /// Used to construct timestamp of a consume power record
    id: String,
    /// Timestamp of a consume power record
    time: String,
    /// Value in Watts
    value: String,
}

impl MeterRecord {
    /// Constructs new record.
    ///
    /// Uses provided simulation seconds to construct datetime from it.
    pub fn new(secs: u32, value: f32) -> Self {
        let time = crate::utils::secs_to_date(secs);
        Self {
            id: secs.to_string(),
            value: value.to_string(),
            time: time.to_string(),
        }
    }

    /// Returns parsed datetime.
    pub fn datetime(&self) -> DateTime<Local> {
        // chrono parsing from it's own types is amazingly flawed
        // But help is on it's way: https://github.com/chronotope/chrono/pull/807
        let time = self.time.clone().replace(" +", "+").replace(' ', "T");

        // 2022-11-17T16:52:34.565738866+02:00
        let dt = DateTime::parse_from_str(&time, "%+").expect("DateTime from string");
        dt.into()
    }

    /// Get hours converted in f32
    pub fn hours(&self) -> f32 {
        let dt = self.datetime();
        dt.hour() as f32 + dt.minute() as f32 / 60.0
    }

    pub fn value(&self) -> f32 {
        self.value.parse().expect("Value INT parsing")
    }
}

impl From<String> for MeterRecord {
    fn from(s: String) -> Self {
        let mut s = s.split('#');
        let id = s.next().expect("No sec").to_owned();
        let time = s.next().expect("No time").to_owned();
        let value = s.next().expect("No value").to_owned();
        Self { id, time, value }
    }
}

impl From<Vec<u8>> for MeterRecord {
    fn from(v: Vec<u8>) -> Self {
        let s = String::from_utf8(v).expect("Not valid set of bytes");
        s.into()
    }
}

impl ToString for MeterRecord {
    fn to_string(&self) -> String {
        format!("{}#{}#{}", self.id, self.time, self.value)
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

#[cfg(test)]
mod tests {
    use crate::{meter::MeterRecord, utils};
    use expect_test::expect;

    #[test]
    fn parse_meter_datetime() {
        let mv = MeterRecord::new(3333, 100.0);
        expect!["3333#2022-11-21 00:55:00 +02:00#100"].assert_eq(&mv.to_string());
        let mv = MeterRecord::new(50000, 100.0);
        expect!["50000#2022-11-21 13:53:00 +02:00#100"].assert_eq(&mv.to_string());
    }

    #[test]
    fn meter_hours() {
        let mv = MeterRecord::new(3333, 100.0);
        let normal = utils::_f32_to_hour_str(mv.hours());
        expect!["0:55"].assert_eq(&normal);
        let mv = MeterRecord::new(50000, 100.0);
        let normal = utils::_f32_to_hour_str(mv.hours());
        expect!["13:52"].assert_eq(&normal);
    }
}
