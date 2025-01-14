//! Time series related operations
use chrono::DateTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Debug)]
pub(crate) enum OutputSize {
    Compact,
    Full,
}

impl OutputSize {
    pub(crate) fn to_string(&self) -> &'static str {
        use self::OutputSize::*;
        match self {
            Compact => "compact",
            Full => "full",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    pub fn to_string(self) -> &'static str {
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
#[derive(Debug, Clone)]
pub struct TimeSeries {
    /// Symbol the time series refers to.
    pub symbol: String,
    /// Date the information was last refreshed at.
    pub last_refreshed: DateTime<Tz>,
    /// Entries in the time series, sorted by ascending dates.
    pub entries: Vec<Entry>,
}

/// Represents a set of values for an equity for a given period in the time series.
#[derive(Debug, PartialEq, Clone)]
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
    /// Adjusted close value.
    pub adjusted_close: Option<f64>,
    /// Dividend amount.
    pub dividend_amount: Option<f64>,
    /// Split coefficient.
    pub split_coefficient: Option<f64>,
}

#[derive(Debug, Clone)]
pub(crate) enum Function {
    IntraDay(IntradayInterval),
    Daily,
    Weekly,
    Monthly,
    DailyAdjusted,
    WeeklyAdjusted,
    MonthlyAdjusted,
}

impl From<&'_ Function> for &'static str {
    fn from(function: &'_ Function) -> Self {
        use Function::*;
        match function {
            IntraDay(_) => "TIME_SERIES_INTRADAY",
            Daily => "TIME_SERIES_DAILY",
            Weekly => "TIME_SERIES_WEEKLY",
            Monthly => "TIME_SERIES_MONTHLY",
            DailyAdjusted => "TIME_SERIES_DAILY_ADJUSTED",
            WeeklyAdjusted => "TIME_SERIES_WEEKLY_ADJUSTED",
            MonthlyAdjusted => "TIME_SERIES_MONTHLY_ADJUSTED",
        }
    }
}

pub(crate) mod parser {
    use super::*;
    use crate::deserialize::{from_str, parse_date};
    use crate::error::Error;
    use chrono_tz::Tz;
    use std::collections::HashMap;
    use std::io::Read;

    pub(crate) enum TimeSeriesHelperEnum {
        Adjusted(TimeSeriesHelper<EntryHelperAdjusted>),
        Regular(TimeSeriesHelper<EntryHelper>),
    }

    impl TimeSeriesHelperEnum {
        pub(crate) fn error(&self) -> Option<&String> {
            match self {
                TimeSeriesHelperEnum::Adjusted(helper) => helper.error.as_ref(),
                TimeSeriesHelperEnum::Regular(helper) => helper.error.as_ref(),
            }
        }

        pub(crate) fn metadata(&self) -> Option<&HashMap<String, String>> {
            match self {
                TimeSeriesHelperEnum::Adjusted(helper) => helper.metadata.as_ref(),
                TimeSeriesHelperEnum::Regular(helper) => helper.metadata.as_ref(),
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub(crate) struct EntryHelper {
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
    pub(crate) struct EntryHelperAdjusted {
        #[serde(rename = "1. open", deserialize_with = "from_str")]
        pub open: f64,
        #[serde(rename = "2. high", deserialize_with = "from_str")]
        pub high: f64,
        #[serde(rename = "3. low", deserialize_with = "from_str")]
        pub low: f64,
        #[serde(rename = "4. close", deserialize_with = "from_str")]
        pub close: f64,
        #[serde(rename = "5. adjusted close", deserialize_with = "from_str")]
        pub adjusted_close: f64,
        #[serde(rename = "6. volume", deserialize_with = "from_str")]
        pub volume: u64,
        #[serde(rename = "7. dividend amount", deserialize_with = "from_str")]
        pub dividend_amount: f64,
        #[serde(
            rename = "8. split coefficient",
            default = "default_split_coefficient",
            deserialize_with = "from_str"
        )]
        pub split_coefficient: f64,
    }

    fn default_split_coefficient() -> f64 {
        1.0
    }

    #[derive(Debug, Deserialize)]
    pub struct TimeSeriesHelper<T> {
        #[serde(rename = "Error Message")]
        pub(crate) error: Option<String>,
        #[serde(rename = "Meta Data")]
        pub(crate) metadata: Option<HashMap<String, String>>,
        #[serde(flatten)]
        pub(crate) time_series: Option<HashMap<String, HashMap<String, T>>>,
    }

    pub(crate) fn parse(function: &Function, reader: impl Read) -> Result<TimeSeries, Error> {
        let helper = match function {
            Function::DailyAdjusted | Function::WeeklyAdjusted | Function::MonthlyAdjusted => {
                let helper: TimeSeriesHelper<EntryHelperAdjusted> =
                    serde_json::from_reader(reader)?;
                TimeSeriesHelperEnum::Adjusted(helper)
            }
            _ => {
                let helper: TimeSeriesHelper<EntryHelper> = serde_json::from_reader(reader)?;
                TimeSeriesHelperEnum::Regular(helper)
            }
        };

        if let Some(error) = helper.error() {
            return Err(Error::APIError(error.clone()));
        }

        let metadata = helper
            .metadata()
            .ok_or_else(|| Error::ParsingError("missing metadata".into()))?;

        let symbol = metadata
            .get("2. Symbol")
            .ok_or_else(|| Error::ParsingError("missing symbol".into()))?
            .to_string();

        let time_zone_key = match function {
            Function::IntraDay(_) => "6. Time Zone",
            Function::Daily | Function::DailyAdjusted => "5. Time Zone",
            Function::Weekly
            | Function::Monthly
            | Function::WeeklyAdjusted
            | Function::MonthlyAdjusted => "4. Time Zone",
        };

        let time_zone: Tz = metadata
            .get(time_zone_key)
            .ok_or_else(|| Error::ParsingError("missing time zone".into()))?
            .parse()
            .map_err(|_| Error::ParsingError("error parsing time zone".into()))?;

        let last_refreshed = metadata
            .get("3. Last Refreshed")
            .ok_or_else(|| Error::ParsingError("missing last refreshed".into()))
            .map(|v| parse_date(v, time_zone))??;

        let time_series_key = match function {
            Function::IntraDay(interval) => format!("Time Series ({})", interval.to_string()),
            Function::Daily => "Time Series (Daily)".to_string(),
            Function::Weekly => "Weekly Time Series".to_string(),
            Function::Monthly => "Monthly Time Series".to_string(),
            Function::DailyAdjusted => "Time Series (Daily)".to_string(),
            Function::WeeklyAdjusted => "Weekly Adjusted Time Series".to_string(),
            Function::MonthlyAdjusted => "Monthly Adjusted Time Series".to_string(),
        };

        let mut entries: Vec<Entry> = vec![];

        match helper {
            TimeSeriesHelperEnum::Adjusted(h) => {
                let time_series_map = h
                    .time_series
                    .ok_or_else(|| Error::ParsingError("missing time series".into()))?;

                let time_series = time_series_map
                    .get(&time_series_key)
                    .ok_or_else(|| Error::ParsingError("missing requested time series".into()))?;

                for (d, v) in time_series.iter() {
                    let date = parse_date(d, time_zone)?;
                    let entry = Entry {
                        date,
                        open: v.open,
                        high: v.high,
                        low: v.low,
                        close: v.close,
                        volume: v.volume,
                        adjusted_close: Some(v.adjusted_close),
                        dividend_amount: Some(v.dividend_amount),
                        split_coefficient: Some(v.split_coefficient),
                    };
                    entries.push(entry);
                }
            }
            TimeSeriesHelperEnum::Regular(h) => {
                let time_series_map = h
                    .time_series
                    .ok_or_else(|| Error::ParsingError("missing time series".into()))?;

                let time_series = time_series_map
                    .get(&time_series_key)
                    .ok_or_else(|| Error::ParsingError("missing requested time series".into()))?;

                for (d, v) in time_series.iter() {
                    let date = parse_date(d, time_zone)?;
                    let entry = Entry {
                        date,
                        open: v.open,
                        high: v.high,
                        low: v.low,
                        close: v.close,
                        volume: v.volume,
                        adjusted_close: None,
                        dividend_amount: None,
                        split_coefficient: None,
                    };
                    entries.push(entry);
                }
            }
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                volume: 4129781,
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
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
                adjusted_close: None,
                dividend_amount: None,
                split_coefficient: None
            }
        );
    }

    #[test]
    fn parse_daily_adjusted() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_daily_adjusted.json");
        let time_series = parser::parse(&Function::DailyAdjusted, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 100);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2024-08-20", Eastern).unwrap(),
                open: 194.59,
                high: 196.21,
                low: 193.75,
                close: 196.03,
                volume: 1790371,
                adjusted_close: Some(194.489652284383),
                dividend_amount: Some(0.0000),
                split_coefficient: Some(1.0)
            }
        );
        assert_eq!(
            time_series.entries[1],
            Entry {
                date: parse_date("2024-08-21", Eastern).unwrap(),
                open: 195.97,
                high: 197.33,
                low: 194.115,
                close: 197.21,
                volume: 2579343,
                adjusted_close: Some(195.660380181621),
                dividend_amount: Some(0.0),
                split_coefficient: Some(1.0)
            }
        );
    }

    #[test]
    fn parse_weekly_adjusted() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_weekly_adjusted.json");
        let time_series = parser::parse(&Function::WeeklyAdjusted, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 16);
        assert_eq!(
            time_series.entries[1],
            Entry {
                date: parse_date("2024-10-11", Eastern).unwrap(),
                open: 225.3800,
                high: 235.8300,
                low: 225.0200,
                close: 233.2600,
                volume: 18398213,
                adjusted_close: Some(231.4271),
                dividend_amount: Some(0.0000),
                split_coefficient: Some(1.0)
            }
        );
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2024-10-04", Eastern).unwrap(),
                open: 220.6500,
                high: 226.0800,
                low: 215.7980,
                close: 226.0000,
                volume: 17778630,
                adjusted_close: Some(224.2242),
                dividend_amount: Some(0.0000),
                split_coefficient: Some(1.0)
            }
        );
    }

    #[test]
    fn parse_monthly_adjusted() {
        let data: &[u8] = include_bytes!("../tests/json/time_series_monthly_adjusted.json");
        let time_series = parser::parse(&Function::MonthlyAdjusted, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 11);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: parse_date("2024-03-28", Eastern).unwrap(),
                open: 185.4900,
                high: 199.1800,
                low: 185.1800,
                close: 190.9600,
                volume: 99921776,
                adjusted_close: Some(185.9534),
                dividend_amount: Some(0.0000),
                split_coefficient: Some(1.0)
            }
        );
        assert_eq!(
            time_series.entries[1],
            Entry {
                date: parse_date("2024-04-30", Eastern).unwrap(),
                open: 190.0000,
                high: 193.2800,
                low: 165.2605,
                close: 166.2000,
                volume: 98297181,
                adjusted_close: Some(161.8426),
                dividend_amount: Some(0.0000),
                split_coefficient: Some(1.0)
            }
        );
    }
}
