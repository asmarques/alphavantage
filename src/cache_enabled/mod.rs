use chrono::{DateTime, FixedOffset};
use chrono_tz::Tz;

pub mod time_series;
pub mod client;

pub(crate) fn tz_datetime_to_fixed_offset_datetime(datetime: DateTime<Tz>) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(std::str::from_utf8(&datetime.to_rfc3339().as_bytes()).unwrap()).unwrap()
}