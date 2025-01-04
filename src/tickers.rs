use chrono::{FixedOffset, NaiveTime};

/// Respresent a set of search results.
#[derive(Debug, Clone)]
pub struct SearchResults {
    /// The query that was searched.
    pub query: Option<String>,
    /// The list of matches, sorted by highest match score to lowest.
    pub entries: Vec<Entry>,
}

/// Represents a set of values for a ticker
#[derive(Debug, PartialEq, Clone)]
pub struct Entry {
    /// Symbol.
    pub symbol: String,
    /// Name.
    pub name: String,
    /// Type.
    pub stock_type: String,
    /// Region.
    pub region: String,
    /// Market open time.
    pub market_open: NaiveTime,
    /// Market close time.
    pub market_close: NaiveTime,
    /// Timezone.
    pub timezone: FixedOffset,
    /// Currency.
    pub currency: String,
    /// Match score.
    pub match_score: f64,
}

pub(crate) mod parser {
    use super::*;
    use crate::deserialize::{from_str, parse_time};
    use crate::error::Error;
    use chrono::FixedOffset;

    use serde::Deserialize;
    use std::io::Read;

    fn parse_offset(offset: &str) -> Option<f64> {
        if let Some(sign) = offset.get(3..4) {
            if let Ok(hours) = offset[4..].parse::<f64>() {
                return match sign {
                    "+" => Some(hours),
                    "-" => Some(-hours),
                    _ => None,
                };
            }
        }
        None
    }

    fn get_utc_offset_from_str(offset: &str) -> Result<FixedOffset, Error> {
        let offset =
            parse_offset(offset).ok_or(Error::ParsingError("error parsing offset".into()))?;

        let offset_hours = offset.trunc() as i32; // Extract the integer part
        let offset_minutes = ((offset.fract() * 60.0).round()) as i32; // Convert fractional part to minutes
        let total_offset_seconds = offset_hours * 3600 + offset_minutes * 60;
        Ok(FixedOffset::east_opt(total_offset_seconds).unwrap())
    }

    #[derive(Debug, Deserialize, Clone)]
    struct EntryHelper {
        #[serde(rename = "1. symbol", deserialize_with = "from_str")]
        symbol: String,
        #[serde(rename = "2. name", deserialize_with = "from_str")]
        name: String,
        #[serde(rename = "3. type", deserialize_with = "from_str")]
        stock_type: String,
        #[serde(rename = "4. region", deserialize_with = "from_str")]
        region: String,
        #[serde(rename = "5. marketOpen", deserialize_with = "from_str")]
        market_open: String,
        #[serde(rename = "6. marketClose", deserialize_with = "from_str")]
        market_close: String,
        #[serde(rename = "7. timezone", deserialize_with = "from_str")]
        timezone: String,
        #[serde(rename = "8. currency", deserialize_with = "from_str")]
        currency: String,
        #[serde(rename = "9. matchScore", deserialize_with = "from_str")]
        match_score: f64,
    }

    #[derive(Debug, Deserialize)]
    struct SearchResultsHelper {
        #[serde(rename = "bestMatches")]
        entries: Vec<EntryHelper>,
    }

    pub fn parse(query: Option<String>, reader: impl Read) -> Result<SearchResults, Error> {
        let helper: SearchResultsHelper = serde_json::from_reader(reader)?;
        let entries: Vec<Result<Entry, Error>> = helper
            .entries
            .clone()
            .into_iter()
            .map(|entry| -> Result<Entry, Error> {
                let timezone = get_utc_offset_from_str(&entry.timezone)?;
                let entry = Entry {
                    symbol: entry.symbol,
                    name: entry.name,
                    stock_type: entry.stock_type,
                    region: entry.region,
                    market_open: parse_time(&entry.market_open)?,
                    market_close: parse_time(&entry.market_close)?,
                    timezone,
                    currency: entry.currency,
                    match_score: entry.match_score,
                };
                Ok(entry)
            })
            .collect();
        // if there is any error, return the error. map to entrys
        if entries.iter().any(|entry| entry.is_err()) {
            return Err(entries.into_iter().find_map(|entry| entry.err()).unwrap());
        }
        let entries: Vec<Entry> = entries.into_iter().map(|entry| entry.unwrap()).collect();
        Ok(SearchResults { query, entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::parse_time;
    use std::io::BufReader;

    #[test]
    fn parse_tesco() {
        let data: &[u8] = include_bytes!("../tests/json/ticker_search_tesco.json");
        let results = parser::parse(None, BufReader::new(data))
            .expect("failed to parse tesco search results");
        assert_eq!(results.query, None);
        assert_eq!(results.entries.len(), 5);
        assert_eq!(
            results.entries[0],
            Entry {
                symbol: "TSCO.LON".into(),
                name: "Tesco PLC".into(),
                stock_type: "Equity".into(),
                region: "United Kingdom".into(),
                market_open: parse_time("08:00").unwrap(),
                market_close: parse_time("16:30").unwrap(),
                timezone: FixedOffset::east_opt(1 * 60 * 60).unwrap(),
                currency: "GBX".into(),
                match_score: 0.7273
            }
        );
    }

    #[test]
    fn parse_tencent() {
        let data: &[u8] = include_bytes!("../tests/json/ticker_search_tencent.json");
        let results = parser::parse(None, BufReader::new(data))
            .expect("failed to parse tencent search results");
        assert_eq!(results.query, None);
        assert_eq!(results.entries.len(), 6);
        assert_eq!(
            results.entries[0],
            Entry {
                symbol: "NNND.FRK".into(),
                name: "Tencent Holdings Ltd".into(),
                stock_type: "Equity".into(),
                region: "Frankfurt".into(),
                market_open: parse_time("08:00").unwrap(),
                market_close: parse_time("20:00").unwrap(),
                timezone: FixedOffset::east_opt(2 * 60 * 60).unwrap(),
                currency: "EUR".into(),
                match_score: 0.5185
            }
        );
    }
}
