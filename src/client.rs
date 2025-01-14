use crate::api::{APIRequest, APIRequestBuilder};
use crate::error::Error;
use crate::time_series;
use crate::{exchange_rate, tickers};
use std::io::Cursor;
use std::io::Read;

/// An asynchronous client for the Alpha Vantage API.
pub struct Client {
    builder: APIRequestBuilder,
    client: reqwest::Client,
}

impl Client {
    /// Create a new client using the specified API `key`.
    pub fn new(key: &str) -> Client {
        Client {
            builder: APIRequestBuilder::new(key),
            client: reqwest::Client::new(),
        }
    }

    /// Retrieve intraday time series for the specified `symbol` updated in realtime (latest 100 data points).
    pub async fn get_time_series_intraday(
        &self,
        symbol: &str,
        interval: time_series::IntradayInterval,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::IntraDay(interval),
            symbol,
            time_series::OutputSize::Compact,
        )
        .await
    }

    /// Retrieve intraday time series for the specified `symbol` updated in realtime (full data set).
    pub async fn get_time_series_intraday_full(
        &self,
        symbol: &str,
        interval: time_series::IntradayInterval,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::IntraDay(interval),
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve daily time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_daily(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::Daily,
            symbol,
            time_series::OutputSize::Compact,
        )
        .await
    }

    /// Retrieve daily time series for the specified `symbol` (full data set).
    pub async fn get_time_series_daily_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::Daily,
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve weekly time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_weekly(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::Weekly,
            symbol,
            time_series::OutputSize::Compact,
        )
        .await
    }

    /// Retrieve weekly time series for the specified `symbol` (full data set).
    pub async fn get_time_series_weekly_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::Weekly,
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve monthly time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_monthly(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::Monthly,
            symbol,
            time_series::OutputSize::Compact,
        )
        .await
    }

    /// Retrieve monthly time series for the specified `symbol` (full data set).
    pub async fn get_time_series_monthly_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::Monthly,
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve daily adjusted time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_daily_adjusted_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::DailyAdjusted,
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve daily adjusted time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_daily_adjusted(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::DailyAdjusted,
            symbol,
            time_series::OutputSize::Compact,
        )
        .await
    }

    /// Retrieve weekly adjusted time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_weekly_adjusted_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::WeeklyAdjusted,
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve monthly adjusted time series for the specified `symbol` (latest 100 data points).
    pub async fn get_time_series_monthly_adjusted_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &time_series::Function::MonthlyAdjusted,
            symbol,
            time_series::OutputSize::Full,
        )
        .await
    }

    /// Retrieve the exchange rate from the currency specified by `from_currency_code` to the
    /// currency specified by `to_currency_code`.
    pub async fn get_exchange_rate(
        &self,
        from_currency_code: &str,
        to_currency_code: &str,
    ) -> Result<exchange_rate::ExchangeRate, Error> {
        let function = "CURRENCY_EXCHANGE_RATE";
        let params = vec![
            ("from_currency", from_currency_code),
            ("to_currency", to_currency_code),
        ];
        let request = self.builder.create(function, &params);
        let response = self.api_call(request).await?;
        let result = exchange_rate::parser::parse(response)?;
        Ok(result)
    }

    /// Retrieve a list of ticker symbols that match the specified `query`.
    pub async fn get_tickers(&self, query: &str) -> Result<tickers::SearchResults, Error> {
        let function = "SYMBOL_SEARCH";
        let params = vec![("keywords", query)];
        let request = self.builder.create(function, &params);
        let response = self.api_call(request).await?;
        let result = tickers::parser::parse(Some(query.to_string()), response)?;
        Ok(result)
    }

    async fn get_time_series(
        &self,
        function: &time_series::Function,
        symbol: &str,
        output_size: time_series::OutputSize,
    ) -> Result<time_series::TimeSeries, Error> {
        let mut params = vec![("symbol", symbol), ("outputsize", output_size.to_string())];
        if let time_series::Function::IntraDay(interval) = function {
            params.push(("interval", interval.to_string()));
        }
        let request = self.builder.create(function.into(), &params);
        let response = self.api_call(request).await?;
        let result = time_series::parser::parse(function, response)?;
        Ok(result)
    }

    async fn api_call(&self, request: APIRequest<'_>) -> Result<impl Read, Error> {
        let response = self.client.execute(request.into()).await?;
        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(Error::ServerError(status.as_u16()));
        }
        let reader = Cursor::new(response.bytes().await?);
        Ok(reader)
    }
}
