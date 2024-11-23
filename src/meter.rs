use chrono::{DateTime, Local, Timelike};
use rand::Rng;

/// Meter payload message with time and amplitude
#[derive(PartialEq, Debug)]
pub(crate) struct MeterRecord {
    /// Timestamp of a consume power record
    time: String,
    /// Value in Watts
    value: String,
}

impl MeterRecord {
    /// Returns parsed datetime.
    pub fn datetime(&self) -> DateTime<Local> {
        // chrono parsing from it's own types is amazingly flawed
        // But help is on it's way: https://github.com/chronotope/chrono/pull/807
        let time = self.time.clone().replace(" +", "+").replace(" ", "T");

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
        let time = s.next().expect("No time").to_owned();
        let value = s.next().expect("No value").to_owned();
        Self { time, value }
    }
}

impl From<Vec<u8>> for MeterRecord {
    fn from(v: Vec<u8>) -> Self {
        let s = String::from_utf8(v).expect("Not valid set of bytes");
        s.into()
    }
}

impl From<f32> for MeterRecord {
    fn from(v: f32) -> Self {
        let time: DateTime<Local> = chrono::Utc::now().into();
        Self {
            time: time.to_string(),
            value: v.to_string(),
        }
    }
}

impl ToString for MeterRecord {
    fn to_string(&self) -> String {
        format!("{}#{}", self.time, self.value)
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
    use crate::meter::MeterRecord;
    use chrono::DateTime;

    #[test]
    fn parse_meter_datetime() {
        const DT: &str = "2022-11-17T16:52:34.565738866+02:00";

        let mv = MeterRecord {
            time: DT.to_owned(),
            value: "100".to_owned(),
        };
        let expected = DateTime::parse_from_str(DT, "%+").unwrap();
        let dt = mv.datetime();

        assert_eq!(dt.to_string(), expected.to_string());
    }

    #[test]
    fn meter_hours() {
        const DT: &str = "2022-11-17T16:52:34.565738866+02:00";
        const EXPECTED: f32 = 16.866667;

        let mv = MeterRecord {
            time: DT.to_owned(),
            value: "100".to_owned(),
        };

        assert_eq!(mv.hours(), EXPECTED);
    }
}
