use crate::api::{APIRequest, APIRequestBuilder};
use crate::error::Error;
use crate::time_series::{Function, IntradayInterval, OutputSize};
use crate::cache_enabled::tickers;
use crate::cache_enabled::exchange_rate;
use crate::cache_enabled::time_series;
use std::io::Cursor;
use std::io::Read;
use disk_cache::cache_async;
use tokio;

/// An asynchronous client for the Alpha Vantage API, using cacheable data structures
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
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_intraday/{symbol}_{interval:?}", invalidate_rate = 1200)]
    pub async fn get_time_series_intraday(
        &self,
        symbol: &str,
        interval: IntradayInterval,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::IntraDay(interval),
            symbol,
            OutputSize::Compact,
        )
        .await
    }

    /// Retrieve intraday time series for the specified `symbol` updated in realtime (full data set).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_intraday_full/{symbol}_{interval:?}", invalidate_rate = 1200)]
    pub async fn get_time_series_intraday_full(
        &self,
        symbol: &str,
        interval: IntradayInterval,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::IntraDay(interval),
            symbol,
            OutputSize::Full,
        )
        .await
    }

    /// Retrieve daily time series for the specified `symbol` (latest 100 data points).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_daily/{symbol}", invalidate_rate = 86400)]
    pub async fn get_time_series_daily(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::Daily,
            symbol,
            OutputSize::Compact,
        )
        .await
    }

    /// Retrieve daily time series for the specified `symbol` (full data set).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_daily_full/{symbol}", invalidate_rate = 86400)]
    pub async fn get_time_series_daily_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::Daily,
            symbol,
            OutputSize::Full,
        )
        .await
    }

    /// Retrieve weekly time series for the specified `symbol` (latest 100 data points).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_weekly/{symbol}", invalidate_rate = 604800)]
    pub async fn get_time_series_weekly(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::Weekly,
            symbol,
            OutputSize::Compact,
        )
        .await
    }

    /// Retrieve weekly time series for the specified `symbol` (full data set).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_weekly_full/{symbol}", invalidate_rate = 604800)]
    pub async fn get_time_series_weekly_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::Weekly,
            symbol,
            OutputSize::Full,
        )
        .await
    }

    /// Retrieve monthly time series for the specified `symbol` (latest 100 data points).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_monthly/{symbol}", invalidate_rate = 2592000)]
    pub async fn get_time_series_monthly(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::Monthly,
            symbol,
            OutputSize::Compact,
        )
        .await
    }

    /// Retrieve monthly time series for the specified `symbol` (full data set).
    #[cache_async(cache_root = "~/.cache/alphavantage/get_time_series_monthly_full/{symbol}", invalidate_rate = 2592000)]
    pub async fn get_time_series_monthly_full(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(
            &Function::Monthly,
            symbol,
            OutputSize::Full,
        )
        .await
    }

    /// Retrieve the exchange rate from the currency specified by `from_currency_code` to the
    /// currency specified by `to_currency_code`.
    #[cache_async(cache_root = "~/.cache/alphavantage/get_exchange_rate/{from_currency_code}_{to_currency_code}", invalidate_rate = 86400)]
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
    #[cache_async(cache_root = "~/.cache/alphavantage/get_tickers/{query}", invalidate_rate = 86400)]
    pub async fn get_tickers(
        &self,
        query: &str,
    ) -> Result<tickers::SearchResults, Error> {
        let function = "SYMBOL_SEARCH";
        let params = vec![("keywords", query)];
        let request = self.builder.create(function, &params);
        let response = self.api_call(request).await?;
        let result = tickers::parser::parse(Some(query.to_string()), response)?;
        Ok(result)
    }

    async fn get_time_series(
        &self,
        function: &Function,
        symbol: &str,
        output_size: OutputSize,
    ) -> Result<time_series::TimeSeries, Error> {
        let mut params = vec![("symbol", symbol), ("outputsize", output_size.to_string())];
        if let Function::IntraDay(interval) = function {
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
