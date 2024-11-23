use crate::SECS;
use chrono::{DateTime, Local};

pub(crate) fn secs_to_date(secs: u32) -> DateTime<Local> {
    let dt: DateTime<Local> = chrono::Utc::now().into();
    let h = (secs as f32 / SECS as f32) * 24.0;
    let (h, m) = (h.floor(), h.fract() * 60.0);
    dt.date().and_hms(h as u32, m as u32, 0)
}

pub(crate) fn _f32_to_hour_str(h: f32) -> String {
    let (h, m) = (h.floor(), h.fract() * 60.0);
    format!("{h}:{}", &m.to_string()[..2])
}
