//! Blocking client implementation
use crate::api::APIRequestBuilder;
use crate::error::Error;
use crate::exchange_rate;
use crate::time_series;
use std::io::Read;

/// A blocking client for the Alpha Vantage API.
pub struct Client {
    builder: APIRequestBuilder,
    client: reqwest::blocking::Client,
}

impl Client {
    /// Create a new blocking client using the specified API `key`.
    pub fn new(key: &str) -> Client {
        Client {
            builder: APIRequestBuilder::new(key),
            client: reqwest::blocking::Client::new(),
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
        let result = exchange_rate::parser::parse(response)?;
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
        let response = self.api_call(function.into(), &params)?;
        let result = time_series::parser::parse(function, response)?;
        Ok(result)
    }

    fn api_call(&self, function: &str, params: &[(&str, &str)]) -> Result<impl Read, Error> {
        let request = self.builder.create(function, params);
        let response = self.client.execute(request.into())?;
        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(Error::ServerError(status.as_u16()));
        }
        Ok(response)
    }
}
