alphavantage
============

[![Build Status](https://travis-ci.org/asmarques/alphavantage.svg)](https://travis-ci.org/asmarques/alphavantage)

A Rust client for the [Alpha Vantage](https://www.alphavantage.co) API.

## Installation

```toml
[dependencies]
alphavantage = "*"
````

## Example

```rust
extern crate alphavantage;

fn main() {
    let client = alphavantage::Client::new("MY_SECRET_TOKEN");
    let time_series = client.get_time_series_daily("GOOG").unwrap();
    let entry = time_series.entries.last().unwrap();
    println!("{:?}", entry);

    let exchange_rate = client.get_exchange_rate("USD", "EUR").unwrap();
    println!("{:?}", exchange_rate);
}
```
