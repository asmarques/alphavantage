# Cached feature

AlphaVantage has limited free tier usibility. The "cache_enabled" feature in this fork relies on the https://github.com/aheschl1/disk_cache_macro
repo, and caches results to reduce unnececarry network calls.

# alphavantage

[![Crate](https://img.shields.io/crates/v/alphavantage.svg)](https://crates.io/crates/alphavantage)
[![Documentation](https://docs.rs/alphavantage/badge.svg)](https://docs.rs/alphavantage)
![Build Status](https://github.com/asmarques/alphavantage/workflows/CI/badge.svg)

A Rust client for the [Alpha Vantage](https://www.alphavantage.co) API.

Currently supports the following operations:

- [TIME_SERIES_INTRADAY](https://www.alphavantage.co/documentation/#intraday)
- [TIME_SERIES_DAILY](https://www.alphavantage.co/documentation/#daily)
- [TIME_SERIES_WEEKLY](https://www.alphavantage.co/documentation/#weekly)
- [TIME_SERIES_MONTHLY](https://www.alphavantage.co/documentation/#monthly)
- [CURRENCY_EXCHANGE_RATE](https://www.alphavantage.co/documentation/#crypto-exchange)
- [SYMBOL_SEARCH](https://www.alphavantage.co/documentation/#symbolsearch)

The default client is asynchronous but a blocking client is also available through the optional `blocking` feature.

## Example

Using the default asynchronous client:

```rust
use alphavantage::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("MY_SECRET_TOKEN");
    let time_series = client.get_time_series_daily("GOOG").await.unwrap();
    let entry = time_series.entries.last().unwrap();
    println!("{:?}", entry);

    let exchange_rate = client.get_exchange_rate("USD", "EUR").await.unwrap();
    println!("{:?}", exchange_rate);
}
```

Using the optional blocking client:

```rust
use alphavantage::blocking::Client;

fn main() {
    let client = Client::new("MY_SECRET_TOKEN");
    let time_series = client.get_time_series_daily("GOOG").unwrap();
    let entry = time_series.entries.last().unwrap();
    println!("{:?}", entry);

    let exchange_rate = client.get_exchange_rate("USD", "EUR").unwrap();
    println!("{:?}", exchange_rate);
}
```
