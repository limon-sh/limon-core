//! A module for managing monitors that are periodically pooled.
//!
//! # Example
//!
//! ```rust, no_run
//! use limon_core::monitor::models::{Config, HttpConfig, PingConfig, Monitor, Measurement};
//!
//! async fn measure_ping() {
//!   let monitor = Monitor {
//!     id: 2,
//!     host: "google.com".into(),
//!     config: Config::Ping(PingConfig {
//!       timeout: 5,
//!       ..Default::default()
//!     })
//!   };
//!
//!   let measure = monitor.measure().await;
//!
//!   assert!(measure.data.is_some() && measure.error.is_none());
//! }
//!
//! # tokio_test::block_on(async {
//! measure_ping().await;
//! # })
//! ```

mod collectors;
mod measure;

pub mod errors;
pub mod models;
