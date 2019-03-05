use crate::exchange_rate;
use crate::time_series;
use failure;
use failure_derive::Fail;
use reqwest;

const URL_ENDPOINT: &str = "https://www.alphavantage.co/query";

/// A client for the Alpha Vantage API.
pub struct Client {
    key: String,
    client: reqwest::Client,
}

/// Set of errors which can occur when calling the API.
#[derive(Debug, Fail)]
pub enum Error {
    /// Error establishing a network connection.
    #[fail(display = "connection error: {}", error)]
    ConnectionError {
        /// Internal error.
        #[cause]
        error: failure::Compat<failure::Error>,
    },
    /// HTTP error returned by the API.
    #[fail(display = "server returned HTTP status code {}", code)]
    ServerError {
        /// HTTP error code.
        code: u16,
    },
    /// Error parsing the result returned from the API.
    #[fail(display = "parsing error: {}", error)]
    ParsingError {
        /// Internal error.
        #[cause]
        error: failure::Compat<failure::Error>,
    },
}

impl Client {
    /// Create a new client using the specified API `key`.
    pub fn new(key: &str) -> Client {
        let client = reqwest::Client::new();
        Client {
            key: String::from(key),
            client,
        }
    }

    /// Retrieve intraday time series for the specified `symbol` updated in realtime.
    pub fn get_time_series_intraday(
        &self,
        symbol: &str,
        interval: time_series::IntradayInterval,
    ) -> Result<time_series::TimeSeries, Error> {
        let function = time_series::Function::IntraDay(interval);
        self.get_time_series(&function, symbol)
    }

    /// Retrieve daily time series for the specified `symbol` including up to 20 years of historical data.
    pub fn get_time_series_daily(&self, symbol: &str) -> Result<time_series::TimeSeries, Error> {
        let function = time_series::Function::Daily;
        self.get_time_series(&function, symbol)
    }

    /// Retrieve weekly time series for the specified `symbol` including up to 20 years of historical data.
    pub fn get_time_series_weekly(&self, symbol: &str) -> Result<time_series::TimeSeries, Error> {
        let function = time_series::Function::Weekly;
        self.get_time_series(&function, symbol)
    }

    /// Retrieve monthly time series for the specified `symbol` including up to 20 years of historical data.
    pub fn get_time_series_monthly(&self, symbol: &str) -> Result<time_series::TimeSeries, Error> {
        let function = time_series::Function::Monthly;
        self.get_time_series(&function, symbol)
    }

    /// Retrieve the exchange rate from the currency specified by `from_currency_code` to the
    /// currency specified by `to_currency_code`.
    pub fn get_exchange_rate(
        &self,
        from_currency_code: &str,
        to_currency_code: &str,
    ) -> Result<exchange_rate::ExchangeRate, Error> {
        let function = "CURRENCY_EXCHANGE_RATE";
        let params = vec![
            ("from_currency", from_currency_code),
            ("to_currency", to_currency_code),
        ];
        let response = self.api_call(function, &params)?;
        let result =
            exchange_rate::parser::parse(response).map_err(|error| Error::ParsingError {
                error: error.compat(),
            })?;
        Ok(result)
    }

    fn get_time_series(
        &self,
        function: &time_series::Function,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        let mut params = vec![("symbol", symbol)];
        if let time_series::Function::IntraDay(interval) = function {
            params.push(("interval", interval.to_string()));
        }
        let response = self.api_call(function.to_string(), &params)?;
        let result = time_series::parser::parse(function, response).map_err(|error| {
            Error::ParsingError {
                error: error.compat(),
            }
        })?;
        Ok(result)
    }

    fn api_call(
        &self,
        function: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, Error> {
        let mut query = vec![("function", function), ("apikey", &self.key)];
        query.extend(params);
        let response = self
            .client
            .get(URL_ENDPOINT)
            .query(&query)
            .send()
            .map_err(|error| Error::ConnectionError {
                error: failure::Error::from(error).compat(),
            })?;
        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(Error::ServerError {
                code: status.as_u16(),
            });
        }
        Ok(response)
    }
}
