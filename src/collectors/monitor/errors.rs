//! A module describing monitor measurement errors.

use crate::models::monitor::metric::{ErrorType, MetricError};

/// Represents all possible errors that can occur during monitoring.
///
/// Wraps specific errors for Ping and HTTP monitors.
#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
  /// An error occurred during a Ping measurement.
  #[error("Ping error: {0}")]
  Ping(#[from] PingError),

  /// An error occurred during an HTTP measurement.
  #[error("HTTP error: {0}")]
  Http(#[from] HttpError),
}

/// Errors that can occur during a Ping measurement.
#[derive(Debug, thiserror::Error)]
pub enum PingError {
  /// DNS resolution failed for the target host.
  #[error("DNS resolve error: {0}")]
  Dns(#[from] trust_dns_resolver::error::ResolveError),

  /// The host did not respond within the timeout.
  #[error("Ping timed out after {timeout:?} seconds")]
  Timeout { timeout: i64 },

  /// The host did not respond.
  #[error("Ping error: {0}")]
  Unknown(#[from] surge_ping::SurgeError),
}

/// Errors that can occur during an HTTP measurement.
#[derive(Debug, thiserror::Error)]
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

impl From<&CollectorError> for MetricError {
  fn from(value: &CollectorError) -> Self {
    match value {
      CollectorError::Ping(error) => MetricError::from(error),
      CollectorError::Http(error) => MetricError::from(error),
    }
  }
}

impl From<&PingError> for MetricError {
  fn from(value: &PingError) -> Self {
    match value {
      PingError::Dns(_) => Self {
        r#type: ErrorType::DnsError,
        details: None,
      },

      PingError::Timeout { timeout: _ } => Self {
        r#type: ErrorType::Timeout,
        details: None,
      },

      PingError::Unknown(error) => match error {
        surge_ping::SurgeError::NetworkError => Self {
          r#type: ErrorType::NetworkError,
          details: None,
        },
        surge_ping::SurgeError::Timeout { seq: _ } => Self {
          r#type: ErrorType::Timeout,
          details: None,
        },
        _ => Self {
          r#type: ErrorType::Unknown,
          details: Some(error.to_string()),
        },
      },
    }
  }
}

impl From<&HttpError> for MetricError {
  fn from(value: &HttpError) -> Self {
    match value {
      HttpError::StatusMismatch {
        expected: _,
        actual: _,
      } => MetricError {
        r#type: ErrorType::StatusMismatch,
        details: None,
      },

      HttpError::KeywordNotFound { keyword: _ } => MetricError {
        r#type: ErrorType::KeywordNotFound,
        details: None,
      },

      HttpError::Unknown(error) => MetricError {
        r#type: ErrorType::Unknown,
        details: Some(error.to_string()),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use rstest::rstest;

  use super::*;

  #[rstest]
  #[case(
    CollectorError::Ping(PingError::Dns(trust_dns_resolver::error::ResolveError::from("test"))),
    ErrorType::DnsError,
    false
  )]
  #[case(
    CollectorError::Ping(PingError::Timeout { timeout: 5 }),
    ErrorType::Timeout,
    false
  )]
  #[case(
    CollectorError::Ping(PingError::Unknown(surge_ping::SurgeError::NetworkError)),
    ErrorType::NetworkError,
    false
  )]
  #[case(
    CollectorError::Ping(PingError::Unknown(surge_ping::SurgeError::Timeout { seq: 1.into() })),
    ErrorType::Timeout,
    false
  )]
  #[case(
    CollectorError::Ping(PingError::Unknown(surge_ping::SurgeError::IncorrectBufferSize)),
    ErrorType::Unknown,
    true
  )]
  #[case(
    CollectorError::Http(HttpError::StatusMismatch { expected: 200, actual: 404 }),
    ErrorType::StatusMismatch,
    false
  )]
  #[case(
  CollectorError::Http(HttpError::KeywordNotFound { keyword: "test".into() }),
    ErrorType::KeywordNotFound,
    false
  )]
  #[case(
    CollectorError::Http(HttpError::Unknown(curl::Error::new(curl_sys::CURLE_COULDNT_CONNECT))),
    ErrorType::Unknown,
    true
  )]
  fn test_collector_error_to_metric_error(
    #[case] error: CollectorError,
    #[case] expected_type: ErrorType,
    #[case] expected_details: bool,
  ) {
    let metric_error: MetricError = (&error).into();

    assert_eq!(metric_error.r#type, expected_type);
    assert_eq!(metric_error.details.is_some(), expected_details);
  }
}
