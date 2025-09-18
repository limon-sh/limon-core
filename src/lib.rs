#![forbid(unsafe_code)]

//! Limon core library.
//!
//! ## Features
//!
//! - **schedule** – Provides the [Schedulable](schedule::Schedulable) trait
//!   and the [Schedule](schedule::Schedule) struct for managing objects that
//!   are polled or executed at regular intervals. Items implementing
//!   [Schedulable](schedule::Schedulable) have a unique `id` and an associated
//!   interval, allowing efficient lookup and grouping.

pub mod schedule;
