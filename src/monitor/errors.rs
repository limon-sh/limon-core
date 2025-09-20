//! A module describing monitor measurement errors.

use thiserror::Error;

/// Represents all possible errors that can occur during monitoring.
///
/// Wraps specific errors for Ping and HTTP monitors.
#[derive(Error, Debug)]
pub enum CollectorError {
  /// An error occurred during a Ping measurement.
  #[error("Ping error: {0}")]
  Ping(#[from] PingError),

  /// An error occurred during an HTTP measurement.
  #[error("HTTP error: {0}")]
  Http(#[from] HttpError),
}

/// Errors that can occur during a Ping measurement.
#[derive(Error, Debug)]
pub enum PingError {
  /// DNS resolution failed for the target host.
  #[error("DNS resolve error: {0}")]
  Dns(#[from] trust_dns_resolver::error::ResolveError),

  /// The host did not respond within the timeout.
  #[error("No reply from {addr:?} timeout")]
  NoReply { addr: String },

  /// The target host is unreachable.
  #[error("The target host is unreachable")]
  Unreachable,
}

/// Errors that can occur during an HTTP measurement.
#[derive(Error, Debug)]
pub enum HttpError {
  /// The HTTP response status code did not match the expected code.
  #[error("Unexpected status code. Expected: {expected:?}, actual: {actual:?}")]
  StatusMismatch { expected: u16, actual: u16 },

  /// The specified keyword was not found in the response body.
  #[error("Keyword '{keyword:?}' not found in response body")]
  KeywordNotFound { keyword: String },

  /// Any other unknown error that occurred during the HTTP request.
  #[error("Unknown error: {0}")]
  Unknown(#[from] curl::Error),
}
