#![forbid(unsafe_code)]

//! Limon core library.
//!
//! This library provides core models, collectors, and utilities for building
//! monitoring system.

extern crate openssl;

pub mod collectors;
pub mod models;
pub mod schedule;
