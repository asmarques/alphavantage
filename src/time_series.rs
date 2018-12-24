use chrono::DateTime;
use chrono_tz::Tz;
use failure::format_err;
use serde::Deserialize;

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
#[derive(Debug)]
pub struct TimeSeries {
    /// Symbol the time series refers to.
    pub symbol: String,
    /// Date the information was last refreshed at.
    pub last_refreshed: DateTime<Tz>,
    /// Entries in the time series, sorted by ascending dates.
    pub entries: Vec<Entry>,
}

/// Represents a set of values for an equity for a given period in the time series.
#[derive(Debug, PartialEq)]
pub struct Entry {
    /// Date.
    pub date: DateTime<Tz>,
    /// Open value.
    pub open: f64,
    /// High value.
    pub high: f64,
    /// Low value.
    pub low: f64,
    /// Close value.
    pub close: f64,
    /// Trading volume.
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
            Monthly => "TIME_SERIES_MONTHLY",
        }
    }
}

pub(crate) mod parser {
    use super::*;
    use crate::deserialize::{from_str, parse_date};
    use chrono_tz::Tz;
    use failure::{err_msg, Error};
    use std::collections::HashMap;
    use std::io::Read;

    #[derive(Debug, Deserialize)]
    struct EntryHelper {
        #[serde(rename = "1. open", deserialize_with = "from_str")]
        pub open: f64,
        #[serde(rename = "2. high", deserialize_with = "from_str")]
        pub high: f64,
        #[serde(rename = "3. low", deserialize_with = "from_str")]
        pub low: f64,
        #[serde(rename = "4. close", deserialize_with = "from_str")]
        pub close: f64,
        #[serde(rename = "5. volume", deserialize_with = "from_str")]
        pub volume: u64,
    }

    #[derive(Debug, Deserialize)]
    pub struct TimeSeriesHelper {
        #[serde(rename = "Error Message")]
        error: Option<String>,
        #[serde(rename = "Meta Data")]
        metadata: Option<HashMap<String, String>>,
        #[serde(flatten)]
        time_series: Option<HashMap<String, HashMap<String, EntryHelper>>>,
    }

    pub(crate) fn parse(function: &Function, reader: impl Read) -> Result<TimeSeries, Error> {
        let helper: TimeSeriesHelper = serde_json::from_reader(reader)?;

        if let Some(error) = helper.error {
            return Err(format_err!("received error: {}", error));
        }

        let metadata = helper.metadata.ok_or_else(|| err_msg("missing metadata"))?;

        let symbol = metadata
            .get("2. Symbol")
            .ok_or_else(|| err_msg("missing symbol"))?
            .to_string();

        let time_zone_key = match function {
            Function::IntraDay(_) => "6. Time Zone",
            Function::Daily => "5. Time Zone",
            Function::Weekly | Function::Monthly => "4. Time Zone",
        };

        let time_zone: Tz = metadata
            .get(time_zone_key)
            .ok_or_else(|| err_msg("missing time zone"))?
            .parse()
            .map_err(|_| err_msg("error parsing time zone"))?;

        let last_refreshed = metadata
            .get("3. Last Refreshed")
            .ok_or_else(|| err_msg("missing last refreshed"))
            .map(|v| parse_date(v, time_zone))??;

        let time_series_key = match function {
            Function::IntraDay(interval) => format!("Time Series ({})", interval.to_string()),
            Function::Daily => "Time Series (Daily)".to_string(),
            Function::Weekly => "Weekly Time Series".to_string(),
            Function::Monthly => "Monthly Time Series".to_string(),
        };

        let time_series_map = helper
            .time_series
            .ok_or_else(|| err_msg("missing time series"))?;

        let time_series = time_series_map
            .get(&time_series_key)
            .ok_or_else(|| err_msg("missing requested time series"))?;

        let mut entries: Vec<Entry> = vec![];

        for (d, v) in time_series.iter() {
            let date = parse_date(d, time_zone)?;
            let entry = Entry {
                date,
                open: v.open,
                high: v.high,
                low: v.low,
                close: v.close,
                volume: v.volume,
            };
            entries.push(entry);
        }

        entries.sort_by_key(|e| e.date);

        let time_series = TimeSeries {
            symbol,
            last_refreshed,
            entries,
        };
        Ok(time_series)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::parse_date;
    use chrono_tz::US::Eastern;
    use std::io::BufReader;

    #[test]
    fn parse_intraday() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_intraday_1min.json");
        let time_series = parser::parse(
            &Function::IntraDay(IntradayInterval::OneMinute),
            BufReader::new(data),
        )
        .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 100);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2018-06-01 14:21:00", Eastern).unwrap(),
                open: 100.3975,
                high: 100.4558,
                low: 100.3850,
                close: 100.4550,
                volume: 67726,
            }
        );
        assert_eq!(
            time_series.entries[99],
            Entry {
                date: parse_date("2018-06-01 16:00:00", Eastern).unwrap(),
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
        assert_eq!(time_series.entries.len(), 100);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2018-01-17", Eastern).unwrap(),
                open: 89.0800,
                high: 90.2800,
                low: 88.7500,
                close: 90.1400,
                volume: 24659472,
            }
        );
        assert_eq!(
            time_series.entries[99],
            Entry {
                date: parse_date("2018-06-08", Eastern).unwrap(),
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
        assert_eq!(time_series.entries.len(), 961);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2000-01-14", Eastern).unwrap(),
                open: 113.4400,
                high: 114.2500,
                low: 101.5000,
                close: 112.2500,
                volume: 157400000,
            }
        );
        assert_eq!(
            time_series.entries[960],
            Entry {
                date: parse_date("2018-06-08", Eastern).unwrap(),
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
        assert_eq!(time_series.entries.len(), 221);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2000-02-29", Eastern).unwrap(),
                open: 98.5000,
                high: 110.0000,
                low: 88.1200,
                close: 89.3700,
                volume: 667243800,
            }
        );
        assert_eq!(
            time_series.entries[220],
            Entry {
                date: parse_date("2018-06-08", Eastern).unwrap(),
                open: 99.2798,
                high: 102.6900,
                low: 99.1700,
                close: 101.6300,
                volume: 150971891,
            }
        );
    }
}
