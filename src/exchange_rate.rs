use chrono::prelude::*;
use chrono_tz::Tz;

/// Represents a currency.
#[derive(Debug, Eq, PartialEq)]
pub struct Currency {
    /// The currency's name.
    pub name: String,
    /// The currency's code. Can be a physical currency using ISO 4217 or a cryptocurrency.
    pub code: String,
}

/// Represents the exchange rate for a currency pair.
#[derive(Debug, PartialEq)]
pub struct ExchangeRate {
    /// Currency to get the exchange rate for.
    pub from: Currency,
    /// Destination currency for the exchange rate.
    pub to: Currency,
    /// Value of the exchange rate.
    pub rate: f64,
    /// Date the exchange rate corresponds to.
    pub date: DateTime<Tz>,
}

pub(crate) mod parser {
    use super::*;
    use deserialize::{from_str, parse_date};
    use failure::{err_msg, Error};
    use serde_json;
    use std::io::Read;

    #[derive(Debug, Deserialize)]
    struct ExchangeRateHelper {
        #[serde(rename = "Realtime Currency Exchange Rate")]
        data: RealtimeExchangeRate,
    }

    #[derive(Debug, Deserialize)]
    struct RealtimeExchangeRate {
        #[serde(rename = "1. From_Currency Code")]
        from_code: String,
        #[serde(rename = "2. From_Currency Name")]
        from_name: String,
        #[serde(rename = "3. To_Currency Code")]
        to_code: String,
        #[serde(rename = "4. To_Currency Name")]
        to_name: String,
        #[serde(rename = "5. Exchange Rate", deserialize_with = "from_str")]
        rate: f64,
        #[serde(rename = "6. Last Refreshed")]
        last_refreshed: String,
        #[serde(rename = "7. Time Zone")]
        time_zone: String,
    }

    pub(crate) fn parse(reader: impl Read) -> Result<ExchangeRate, Error> {
        let helper: ExchangeRateHelper = serde_json::from_reader(reader)?;

        let time_zone: Tz = helper
            .data
            .time_zone
            .parse()
            .map_err(|_| err_msg("error parsing time zone"))?;

        let date = parse_date(&helper.data.last_refreshed, time_zone)?;

        let exchange_rate = ExchangeRate {
            from: Currency {
                name: helper.data.from_name,
                code: helper.data.from_code,
            },
            to: Currency {
                name: helper.data.to_name,
                code: helper.data.to_code,
            },
            rate: helper.data.rate,
            date,
        };
        Ok(exchange_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::UTC;
    use deserialize::parse_date;
    use std::io::BufReader;

    #[test]
    fn parse() {
        let data: &[u8] = include_bytes!("../tests/json/currency_exchange_rate.json");
        let exchange_rate =
            parser::parse(BufReader::new(data)).expect("failed to parse exchange rate");
        assert_eq!(
            exchange_rate,
            ExchangeRate {
                from: Currency {
                    name: "Euro".to_string(),
                    code: "EUR".to_string(),
                },
                to: Currency {
                    name: "United States Dollar".to_string(),
                    code: "USD".to_string(),
                },
                rate: 1.16665014,
                date: parse_date("2018-06-23 10:27:49", UTC).unwrap(),
            }
        );
    }
}
