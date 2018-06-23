use deserialize::from_str;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
/// Represents the interval for an intraday time series.
pub enum IntradayInterval {
    /// 1 minute.
    OneMinute,
    /// 5 minutes.
    FiveMinutes,
    /// 15 minutes.
    FifteenMinutes,
    /// 30 minutes.
    ThirtyMinutes,
    /// 60 minutes.
    SixtyMinutes,
}

impl IntradayInterval {
    pub(crate) fn to_string(self) -> &'static str {
        use self::IntradayInterval::*;
        match self {
            OneMinute => "1min",
            FiveMinutes => "5min",
            FifteenMinutes => "15min",
            ThirtyMinutes => "30min",
            SixtyMinutes => "60min",
        }
    }
}

/// Represents a time series for a given symbol.
#[derive(Debug, Deserialize)]
pub struct TimeSeries {
    #[serde(flatten)]
    entries: HashMap<String, Entry>,
}

impl TimeSeries {
    /// Returns the entries in the time series sorted by ascending date.
    /// Each item corresponds to a pair of the date and corresponding entry.
    pub fn entries(&self) -> Vec<(&String, &Entry)> {
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }

    /// Returns the length of the time series.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the time series is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Represents a set of values for an equity for a given period in the time series.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Entry {
    /// Open value.
    #[serde(rename = "1. open", deserialize_with = "from_str")]
    pub open: f64,
    #[serde(rename = "2. high", deserialize_with = "from_str")]
    /// High value.
    pub high: f64,
    #[serde(rename = "3. low", deserialize_with = "from_str")]
    /// Low value.
    pub low: f64,
    /// Close value.
    #[serde(rename = "4. close", deserialize_with = "from_str")]
    pub close: f64,
    /// Trading volume.
    #[serde(rename = "5. volume", deserialize_with = "from_str")]
    pub volume: u64,
}

#[derive(Debug, Clone)]
pub(crate) enum Function {
    IntraDay(IntradayInterval),
    Daily,
    Weekly,
    Monthly,
}

impl Function {
    pub(crate) fn to_string(&self) -> &'static str {
        use self::Function::*;
        match self {
            IntraDay(_) => "TIME_SERIES_INTRADAY",
            Daily => "TIME_SERIES_DAILY",
            Weekly => "TIME_SERIES_WEEKLY",
            Monthly => "TIME_SERIES_MONTLY",
        }
    }
}

pub(crate) mod parser {
    use super::*;
    use failure::{err_msg, Error};
    use serde_json;
    use serde_json::Value;
    use std::io::Read;

    pub(crate) fn parse(function: &Function, reader: impl Read) -> Result<TimeSeries, Error> {
        let mut object: Value = serde_json::from_reader(reader)?;
        let time_series_key = match function {
            Function::IntraDay(interval) => format!("Time Series ({})", interval.to_string()),
            Function::Daily => "Time Series (Daily)".to_string(),
            Function::Weekly => "Weekly Time Series".to_string(),
            Function::Monthly => "Monthly Time Series".to_string(),
        };

        let time_series_value = object
            .get_mut(time_series_key)
            .ok_or_else(|| err_msg("missing time series entries"))?
            .take();

        let time_series: TimeSeries = serde_json::from_value(time_series_value)?;
        Ok(time_series)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn parse_intraday() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_intraday_1min.json");
        let time_series = parser::parse(
            &Function::IntraDay(IntradayInterval::OneMinute),
            BufReader::new(data),
        ).expect("failed to parse entries");
        assert_eq!(time_series.len(), 100);
        let entries = time_series.entries();
        let (first_time, first_entry) = entries[0];
        assert_eq!(first_time, "2018-06-01 14:21:00");
        assert_eq!(
            first_entry,
            &Entry {
                open: 100.3975,
                high: 100.4558,
                low: 100.3850,
                close: 100.4550,
                volume: 67726,
            }
        );
        let (last_time, last_entry) = entries[99];
        assert_eq!(last_time, "2018-06-01 16:00:00");
        assert_eq!(
            last_entry,
            &Entry {
                open: 100.6150,
                high: 100.8100,
                low: 100.5900,
                close: 100.7900,
                volume: 4129781
            }
        );
    }

    #[test]
    fn parse_daily() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_daily.json");
        let time_series =
            parser::parse(&Function::Daily, BufReader::new(data)).expect("failed to parse entries");
        assert_eq!(time_series.len(), 100);
        let entries = time_series.entries();
        let (first_time, first_entry) = entries[0];
        assert_eq!(first_time, "2018-01-17");
        assert_eq!(
            first_entry,
            &Entry {
                open: 89.0800,
                high: 90.2800,
                low: 88.7500,
                close: 90.1400,
                volume: 24659472,
            }
        );
        let (last_time, last_entry) = entries[99];
        assert_eq!(last_time, "2018-06-08");
        assert_eq!(
            last_entry,
            &Entry {
                open: 101.0924,
                high: 101.9500,
                low: 100.5400,
                close: 101.6300,
                volume: 22165128,
            }
        );
    }

    #[test]
    fn parse_weekly() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_weekly.json");
        let time_series = parser::parse(&Function::Weekly, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.len(), 961);
        let entries = time_series.entries();
        let (first_time, first_entry) = entries[0];
        assert_eq!(first_time, "2000-01-14");
        assert_eq!(
            first_entry,
            &Entry {
                open: 113.4400,
                high: 114.2500,
                low: 101.5000,
                close: 112.2500,
                volume: 157400000,
            }
        );
        let (last_time, last_entry) = entries[960];
        assert_eq!(last_time, "2018-06-08");
        assert_eq!(
            last_entry,
            &Entry {
                open: 101.2600,
                high: 102.6900,
                low: 100.3800,
                close: 101.6300,
                volume: 122316267,
            }
        );
    }

    #[test]
    fn parse_monthly() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_monthly.json");
        let time_series = parser::parse(&Function::Monthly, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.len(), 221);
        let entries = time_series.entries();
        let (first_time, first_entry) = entries[0];
        assert_eq!(first_time, "2000-02-29");
        assert_eq!(
            first_entry,
            &Entry {
                open: 98.5000,
                high: 110.0000,
                low: 88.1200,
                close: 89.3700,
                volume: 667243800,
            }
        );
        let (last_time, last_entry) = entries[220];

        assert_eq!(last_time, "2018-06-08");
        assert_eq!(
            last_entry,
            &Entry {
                open: 99.2798,
                high: 102.6900,
                low: 99.1700,
                close: 101.6300,
                volume: 150971891,
            }
        );
    }
}
