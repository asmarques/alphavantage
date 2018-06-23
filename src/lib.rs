extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
#[macro_use]
extern crate failure;

mod client;
mod deserialize;

pub mod exchange_rate;
pub mod time_series;
pub use client::Client;
