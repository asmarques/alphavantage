//! Time series related operations
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// Represents a time series for a given symbol.
/// 
/// Uses FixedOffset for the date to allow for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    /// Symbol the time series refers to.
    pub symbol: String,
    /// Date the information was last refreshed at.
    pub last_refreshed: DateTime<FixedOffset>,
    /// Entries in the time series, sorted by ascending dates.
    pub entries: Vec<Entry>,
}

/// Represents a set of values for an equity for a given period in the time series.
/// 
/// Uses FixedOffset for the date to allow for serialization.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Date.
    pub date: DateTime<FixedOffset>,
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
    #[serde(default)]
    pub adjusted_close: Option<f64>,
    /// Dividend amount.
    #[serde(default)]
    pub dividend_amount: Option<f64>,
    /// Split coefficient.
    #[serde(default)]
    pub split_coefficient: Option<f64>,
}

pub(crate) mod parser {
    use super::*;
    use crate::cache_enabled::tz_datetime_to_fixed_offset_datetime;
    use crate::deserialize::parse_date;
    use crate::error::Error;
    use crate::time_series::parser::{EntryHelper, EntryHelperAdjusted, TimeSeriesHelper, TimeSeriesHelperEnum};
    use crate::time_series::Function;
    use chrono_tz::Tz;
    use std::io::Read;

    pub(crate) fn parse(function: &Function, reader: impl Read) -> Result<TimeSeries, Error> {
        let helper = match function {
            Function::DailyAdjusted | Function::WeeklyAdjusted | Function::MonthlyAdjusted => {
                let helper: TimeSeriesHelper<EntryHelperAdjusted> = serde_json::from_reader(reader)?;
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
            Function::Weekly | Function::Monthly | Function::WeeklyAdjusted | Function::MonthlyAdjusted => "4. Time Zone",
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
                Function::MonthlyAdjusted => "Monthly Adjusted Time Series".to_string()
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
                    let date = tz_datetime_to_fixed_offset_datetime(parse_date(d, time_zone)?);
                    let entry = Entry {
                        date,
                        open: v.open,
                        high: v.high,
                        low: v.low,
                        close: v.close,
                        volume: v.volume,
                        adjusted_close: Some(v.adjusted_close),
                        dividend_amount: Some(v.dividend_amount),
                        split_coefficient: Some(v.split_coefficient)
                    };
                    entries.push(entry);
                }
            },
            TimeSeriesHelperEnum::Regular(h) => {
                let time_series_map = h
                .time_series
                .ok_or_else(|| Error::ParsingError("missing time series".into()))?;
        
                let time_series = time_series_map
                    .get(&time_series_key)
                    .ok_or_else(|| Error::ParsingError("missing requested time series".into()))?;
        
                for (d, v) in time_series.iter() {
                    let date = tz_datetime_to_fixed_offset_datetime(parse_date(d, time_zone)?);
                    let entry = Entry {
                        date,
                        open: v.open,
                        high: v.high,
                        low: v.low,
                        close: v.close,
                        volume: v.volume,
                        adjusted_close: None,
                        dividend_amount: None,
                        split_coefficient: None
                    };
                    entries.push(entry);
                }
            }
        }

        entries.sort_by_key(|e| e.date);

        let time_series = TimeSeries {
            symbol,
            last_refreshed: tz_datetime_to_fixed_offset_datetime(last_refreshed),
            entries,
        };
        Ok(time_series)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cache_enabled::tz_datetime_to_fixed_offset_datetime, deserialize::parse_date, time_series::{Function, IntradayInterval}};
    use chrono_tz::US::Eastern;
    use std::io::BufReader;

    #[test]
    fn parse_intraday() {
        let data: &[u8] = include_bytes!("../../tests/json/time_series_intraday_1min.json");
        let time_series = parser::parse(
            &Function::IntraDay(IntradayInterval::OneMinute),
            BufReader::new(data),
        )
        .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 100);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2018-06-01 14:21:00", Eastern).unwrap()),
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
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2018-06-01 16:00:00", Eastern).unwrap()),
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
        let data: &[u8] = include_bytes!("../../tests/json/time_series_daily.json");
        let time_series =
            parser::parse(&Function::Daily, BufReader::new(data)).expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 100);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2018-01-17", Eastern).unwrap()),
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
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2018-06-08", Eastern).unwrap()),
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
        let data: &[u8] = include_bytes!("../../tests/json/time_series_weekly.json");
        let time_series = parser::parse(&Function::Weekly, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 961);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2000-01-14", Eastern).unwrap()),
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
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2018-06-08", Eastern).unwrap()),
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
        let data: &[u8] = include_bytes!("../../tests/json/time_series_monthly.json");
        let time_series = parser::parse(&Function::Monthly, BufReader::new(data))
            .expect("failed to parse entries");
        assert_eq!(time_series.entries.len(), 221);
        assert_eq!(
            time_series.entries[0],
            Entry {
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2000-02-29", Eastern).unwrap()),
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
                date: tz_datetime_to_fixed_offset_datetime(parse_date("2018-06-08", Eastern).unwrap()),
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
}