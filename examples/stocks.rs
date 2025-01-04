use alphavantage::time_series::IntradayInterval;
use alphavantage::Client;
use clap::Parser;
use std::env;

const TOKEN_ENV_KEY: &str = "ALPHAVANTAGE_TOKEN";

#[derive(Parser, Debug)]
#[command(about = "Get stock prices.")]
struct Cli {
    #[arg(
        short,
        long,
        help = "API token (ALPHAVANTAGE_TOKEN env var can be used instead)"
    )]
    token: Option<String>,
    #[arg(help = "Period (1min, 5min, 15min, 30min, hourly, daily, weekly or monthly)")]
    period: String,
    #[arg(help = "Stock symbol (e.g. AAPL)")]
    symbol: String,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let token = args
        .token
        .or_else(|| env::var(TOKEN_ENV_KEY).ok())
        .ok_or("missing token")?;

    let symbol = &args.symbol;
    let client = Client::new(&token);

    let tickers = client.get_tickers(symbol).await?;
    let ticker = tickers
        .entries
        .into_iter()
        .find(|t| t.symbol == *symbol)
        .ok_or_else(|| format!("ticker for symbol not found: {}", symbol))?;

    println!("{}: {}", ticker.symbol, ticker.name);
    println!(
        "{} - {} ({})",
        ticker.stock_type, ticker.region, ticker.currency
    );

    let time_series = match args.period.as_str() {
        "1min" => {
            client
                .get_time_series_intraday(symbol, IntradayInterval::OneMinute)
                .await
        }
        "5min" => {
            client
                .get_time_series_intraday(symbol, IntradayInterval::FiveMinutes)
                .await
        }
        "15min" => {
            client
                .get_time_series_intraday(symbol, IntradayInterval::FifteenMinutes)
                .await
        }
        "30min" => {
            client
                .get_time_series_intraday(symbol, IntradayInterval::ThirtyMinutes)
                .await
        }
        "hourly" => {
            client
                .get_time_series_intraday(symbol, IntradayInterval::SixtyMinutes)
                .await
        }
        "daily" => client.get_time_series_daily(symbol).await,
        "weekly" => client.get_time_series_weekly(symbol).await,
        "monthly" => client.get_time_series_monthly(symbol).await,
        _ => return Err(format!("unknown period {}", args.period).into()),
    }?;

    println!("Updated: {}\n", time_series.last_refreshed);
    for entry in time_series.entries {
        let date = match args.period.as_str() {
            "daily" | "weekly" | "monthly" => entry.date.format("%Y-%m-%d"),
            _ => entry.date.format("%Y-%m-%d %H:%M %:z"),
        };
        println!(
            "{}: open = {}, close = {}, high = {}, low = {}, volume = {}",
            date, entry.open, entry.close, entry.high, entry.low, entry.volume
        );
    }
    Ok(())
}
