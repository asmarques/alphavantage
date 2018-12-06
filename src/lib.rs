extern crate chrono;
extern crate chrono_tz;
extern crate failure;
extern crate failure_derive;
extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod client;
mod deserialize;

pub mod exchange_rate;
pub mod time_series;
pub use crate::client::Client;
