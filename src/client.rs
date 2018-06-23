use failure::Error;
use reqwest;
use time_series;

const URL_ENDPOINT: &str = "https://www.alphavantage.co/query";

/// A client for the Alpha Vantage API.
pub struct Client {
    key: String,
    client: reqwest::Client,
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

    /// Retrieve weekly time series for the specified `symbol` including up to 20 years of historical data..
    pub fn get_time_series_weekly(&self, symbol: &str) -> Result<time_series::TimeSeries, Error> {
        let function = time_series::Function::Weekly;
        self.get_time_series(&function, symbol)
    }

    /// Retrieve montly time series for the specified `symbol` including up to 20 years of historical data..
    pub fn get_time_series_montly(&self, symbol: &str) -> Result<time_series::TimeSeries, Error> {
        let function = time_series::Function::Monthly;
        self.get_time_series(&function, symbol)
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
        let result = time_series::parse(function, response)?;
        Ok(result)
    }

    fn api_call(
        &self,
        function: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut query = vec![("function", function), ("apikey", &self.key)];
        query.extend(params);
        self.client.get(URL_ENDPOINT).query(&query).send()
    }
}
