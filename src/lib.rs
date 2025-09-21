#![forbid(unsafe_code)]

//! Limon core library.
//!
//! - **monitor** - Provides abstractions for collecting measurements
//!   from different types of monitoring sources (e.g., network pings, http
//!   endpoints). Each monitor implements the `measure` method, which returns
//!   a [`Measurement`](monitor::models::Measurement) result.
//!
//! - **schedule** – Provides the [`Schedulable`](schedule::Schedulable) trait
//!   and the [`Schedule`](schedule::Schedule) struct for managing objects that
//!   are polled or executed at regular intervals. Items implementing
//!   [`Schedulable`](schedule::Schedulable) have a unique `id` and an associated
//!   interval, allowing efficient lookup and grouping.

pub mod monitor;
pub mod schedule;
