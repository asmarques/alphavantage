extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate chrono_tz;
extern crate failure;

mod client;
mod deserialize;

pub mod exchange_rate;
pub mod time_series;
pub use client::Client;
