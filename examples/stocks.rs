use alphavantage::time_series::IntradayInterval;
use alphavantage::Client;
use std::env;
use structopt::StructOpt;

const TOKEN_ENV_KEY: &str = "ALPHAVANTAGE_TOKEN";

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        short = "t",
        long = "token",
        help = "API token (can use the ALPHAVANTAGE_TOKEN env var instead)"
    )]
    token: Option<String>,
    #[structopt(help = "period (1min, 5min, 15min, 30min, hourly, daily, weekly or monthly)")]
    period: String,
    #[structopt(help = "stock symbol (e.g. AAPL)")]
    symbol: String,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let token = args
        .token
        .or_else(|| env::var(TOKEN_ENV_KEY).ok())
        .ok_or("missing token")?;

    let symbol = &args.symbol;
    let client = Client::new(&token);

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

    println!(
        "{} (updated: {})\n",
        time_series.symbol, time_series.last_refreshed
    );
    for entry in time_series.entries {
        println!("{}:", entry.date);
        println!("  open: {}", entry.open);
        println!("  high: {}", entry.high);
        println!("  low: {}", entry.low);
        println!("  close: {}", entry.close);
        println!("  volume: {}", entry.volume);
    }
    Ok(())
}
