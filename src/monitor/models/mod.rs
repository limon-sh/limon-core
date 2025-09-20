//! A module containing a set of models for monitor measurement.

mod measurement;
mod monitor;

pub use measurement::{Data, HttpData, Measurement, PingData};
pub use monitor::{Config, Header, HttpConfig, Monitor, PingConfig};
