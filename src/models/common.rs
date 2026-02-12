//! A module containing a set of common models.

use time::OffsetDateTime;

use crate::models::monitor::{Data, Measurement};

/// Represents the supported geographical regions for monitoring.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, strum::Display, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "region", rename_all = "lowercase")]
pub enum Region {
  Europe,
  America,
  Asia,
  Australia,
}

/// Represents the target of system metric.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "target", rename_all = "lowercase")]
pub enum Target {
  Monitor,
  #[serde(rename = "monitor:test")]
  #[sqlx(rename = "monitor:test")]
  MonitorTest,
  Heartbeat,
  #[serde(rename = "heartbeat:test")]
  #[sqlx(rename = "heartbeat:test")]
  HeartbeatTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "system_metric")]
pub enum SystemMetricName {
  /// Base availability metric `monitor.up`
  ///
  /// Indicates the availability of the monitor at the time of measurement
  /// (1 - was available, 0 - was not).
  #[serde(rename = "monitor.up")]
  #[sqlx(rename = "monitor.up")]
  MonitorUp,

  /// Base availability metric `heartbeat.up`
  ///
  /// Indicates the availability of the heartbeat
  /// (0 - was available, other - was not).
  #[serde(rename = "heartbeat.up")]
  #[sqlx(rename = "heartbeat.up")]
  HeartbeatUp,

  /// Expiration metric `monitor.ssl.days_left`
  ///
  /// Indicates how many days remain until the certificate expires.
  #[serde(rename = "monitor.ssl.days_left")]
  #[sqlx(rename = "monitor.ssl.days_left")]
  MonitorSslDaysLeft,

  /// Expiration metric `monitor.domain.days_left`
  ///
  /// Indicates how many days remain until the domain name expires.
  #[serde(rename = "monitor.domain.days_left")]
  #[sqlx(rename = "monitor.domain.days_left")]
  MonitorDomainDaysLeft,
}

/// Defines error codes describing the outcome of a monitor execution.
///
/// These codes are used to classify the known reason of an error.
/// [`ErrorType::Unknown`] represents an unclassified error for which
/// detailed information should be preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "error_type")]
#[sqlx(rename_all = "snake_case")]
pub enum ErrorType {
  /// A DNS resolution error occurred.
  DnsError,

  /// The operation exceeded the configured time limit.
  Timeout,

  /// The received response status did not match the expected value.
  StatusMismatch,

  /// The expected keyword was not found in the response body.
  KeywordNotFound,

  /// A network-level error occurred while establishing a connection.
  NetworkError,

  /// An unknown or unclassified error.
  Unknown,
}

/// Represents a single metric event produced by a target at
/// a concrete point in time.
#[derive(Debug)]
pub struct SystemMetric {
  /// Unix timestamp when the metric was created.
  pub timestamp: OffsetDateTime,

  /// Unique identifier of the target that produced this metric.
  pub target_id: i64,

  /// Name of the target that produced this metric.
  pub target: Target,

  /// The name of metric being reported.
  pub name: SystemMetricName,

  /// Numeric value of the metric.
  pub value: f32,

  /// Type of error associated with this metric.
  pub error: MetricError,
}

/// Represents the presence or absence of an error during metric collection.
#[derive(Debug)]
pub struct MetricError {
  /// Type of error by default is None.
  pub r#type: Option<ErrorType>,

  /// Optional detailed description of the error.
  pub details: Option<String>,
}

impl From<&Measurement> for SystemMetric {
  fn from(measurement: &Measurement) -> Self {
    let make = |name: SystemMetricName, value: f32| SystemMetric {
      target: Target::Monitor,
      target_id: measurement.monitor_id,
      timestamp: measurement.timestamp,
      name,
      value,
      error: measurement
        .error
        .as_ref()
        .map(Into::into)
        .unwrap_or_default(),
    };

    if measurement.error.is_some() {
      make(SystemMetricName::MonitorUp, 0f32)
    } else {
      match measurement.data.as_ref() {
        Some(Data::Ping(_)) => make(SystemMetricName::MonitorUp, 1f32),
        Some(Data::Http(_)) => make(SystemMetricName::MonitorUp, 1f32),
        _ => unimplemented!("Measurement data is not implemented"),
      }
    }
  }
}

impl Default for MetricError {
  fn default() -> Self {
    Self {
      r#type: None,
      details: None,
    }
  }
}

#[cfg(test)]
mod tests {
  use rstest::rstest;
  use serde_json;
  use static_assertions::assert_impl_all;
  use time::OffsetDateTime;

  use super::*;
  use crate::collectors::monitor::errors::{CollectorError, PingError};
  use crate::models::common::Region;
  use crate::models::monitor::measurement::{HttpData, PingData};

  assert_impl_all!(SystemMetricName: Clone, Copy);
  assert_impl_all!(ErrorType: Clone, Copy);

  assert_impl_all!(Region: Clone, Copy);
  assert_impl_all!(Region: serde::Serialize, serde::Deserialize<'static>);

  #[test]
  fn test_region_to_string() {
    let cases: Vec<(Region, &str)> = vec![
      (Region::Europe, "europe"),
      (Region::America, "america"),
      (Region::Asia, "asia"),
      (Region::Australia, "australia"),
    ];

    for (region, expected) in cases {
      assert_eq!(region.to_string(), expected);
    }
  }

  #[rstest]
  #[case(SystemMetricName::MonitorUp, "monitor.up")]
  #[case(SystemMetricName::HeartbeatUp, "heartbeat.up")]
  #[case(SystemMetricName::MonitorSslDaysLeft, "monitor.ssl.days_left")]
  #[case(SystemMetricName::MonitorDomainDaysLeft, "monitor.domain.days_left")]
  fn test_metric_name_serialization(#[case] metric: SystemMetricName, #[case] expected_str: &str) {
    let json = serde_json::to_string(&metric).unwrap();
    assert_eq!(json, format!("\"{}\"", expected_str));

    let parsed: SystemMetricName = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, metric);
  }

  #[rstest]
  #[case(ErrorType::DnsError, "dns_error")]
  #[case(ErrorType::Timeout, "timeout")]
  #[case(ErrorType::StatusMismatch, "status_mismatch")]
  #[case(ErrorType::KeywordNotFound, "keyword_not_found")]
  #[case(ErrorType::NetworkError, "network_error")]
  #[case(ErrorType::Unknown, "unknown")]
  fn test_error_type_serialization(#[case] value: ErrorType, #[case] expected_str: &str) {
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, format!("\"{}\"", expected_str));

    let parsed: ErrorType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, value);
  }

  #[rstest]
  #[case(
    Measurement {
      monitor_id: 1,
      timestamp: OffsetDateTime::now_utc(),
      data: Some(Data::Ping(PingData { dns: 10.0, rtt: 20.0 })),
      error: None,
    },
      (SystemMetricName::MonitorUp, 1.0)
  )]
  #[case(
    Measurement {
      monitor_id: 2,
      timestamp: OffsetDateTime::now_utc(),
      data: Some(Data::Http(HttpData {
        dns: 15.0, tcp: 25.0, tls: 35.0, ttfb: 45.0, transfer: 55.0
      })),
      error: None,
    },
      (SystemMetricName::MonitorUp, 1.0)
  )]
  #[case(
    Measurement {
      monitor_id: 3,
      timestamp: OffsetDateTime::now_utc(),
      data: None,
      error: Some(CollectorError::Ping(PingError::Timeout { timeout: 5 })),
    },
    (SystemMetricName::MonitorUp, 0f32)
  )]
  fn test_measurement_to_metrics(
    #[case] measurement: Measurement,
    #[case] expected: (SystemMetricName, f32),
  ) {
    let (expected_name, expected_value) = expected;
    let metric: SystemMetric = (&measurement).into();

    assert_eq!(metric.name, expected_name);
    assert_eq!(metric.value, expected_value);
  }

  #[test]
  fn test_metric_error_default() {
    let value = MetricError::default();

    assert!(value.r#type.is_none());
    assert!(value.details.is_none());
  }
}
