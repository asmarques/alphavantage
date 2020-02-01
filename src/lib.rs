mod client;
mod deserialize;
mod error;

pub mod exchange_rate;
pub mod time_series;
pub use crate::client::Client;
pub use crate::error::Error;
