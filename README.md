alphavantage
============

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
    let entries = time_series.entries();
    let (date, entry) = entries.last().unwrap();
    println!("{}: {}", date, entry.close);
}
```
