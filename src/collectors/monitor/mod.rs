//! A module containing sets of collectors for monitor.

mod dns;
pub mod errors;
mod http;
mod ping;

pub use crate::collectors::monitor::http::Http;
pub use crate::collectors::monitor::ping::Ping;
