//! # alphavantage
//!
//! A Rust client for the [Alpha Vantage](https://www.alphavantage.co) API.
//!
//! Currently supports the following operations:
//! - [TIME_SERIES_INTRADAY](https://www.alphavantage.co/documentation/#intraday)
//! - [TIME_SERIES_DAILY](https://www.alphavantage.co/documentation/#daily)
//! - [TIME_SERIES_WEEKLY](https://www.alphavantage.co/documentation/#weekly)
//! - [TIME_SERIES_MONTHLY](https://www.alphavantage.co/documentation/#monthly)
//! - [CURRENCY_EXCHANGE_RATE](https://www.alphavantage.co/documentation/#crypto-exchange)
//!
//! The default [Client] is asynchronous but a
//! blocking client is also available through the optional `blocking` feature.

mod api;
mod client;
mod deserialize;
mod error;

#[cfg(feature = "blocking")]
pub mod blocking;
pub mod exchange_rate;
pub mod time_series;
pub use crate::client::Client;
pub use crate::error::Error;
