use time::OffsetDateTime;

use crate::models::monitor::measurement::{Data, Measurement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "monitor_metric")]
pub enum MetricName {
  /// Base availability metric `monitor.up`
  ///
  /// Indicates the availability of the monitor at the time of measurement
  /// (1 - was available, 0 - was not).
  #[serde(rename = "monitor.up")]
  #[sqlx(rename = "monitor.up")]
  MonitorUp,

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

  /// Ping metric `monitor.ping.dns`
  ///
  /// Indicates how quickly the domain resolution was processed.
  #[serde(rename = "monitor.ping.dns")]
  #[sqlx(rename = "monitor.ping.dns")]
  MonitorPingDns,

  /// Ping metric `monitor.ping.rtt`
  ///
  /// Indicates how long it took for the ICMP request and response.
  #[serde(rename = "monitor.ping.rtt")]
  #[sqlx(rename = "monitor.ping.rtt")]
  MonitorPingRtt,

  /// Http metric `monitor.http.dns`
  ///
  /// Indicates how quickly the domain resolution was processed.
  #[serde(rename = "monitor.http.dns")]
  #[sqlx(rename = "monitor.http.dns")]
  MonitorHttpDns,

  /// Http metric `monitor.http.tcp`
  ///
  /// Indicates how quickly the TCP connection was established.
  #[serde(rename = "monitor.http.tcp")]
  #[sqlx(rename = "monitor.http.tcp")]
  MonitorHttpTcp,

  /// Http metric `monitor.http.tls`
  ///
  /// Indicates how quickly TLS encryption was established.
  #[serde(rename = "monitor.http.tls")]
  #[sqlx(rename = "monitor.http.tls")]
  MonitorHttpTls,

  /// Http metric `monitor.http.ttfb`
  ///
  /// Indicates how long the server took to process the request and
  /// return the response.
  #[serde(rename = "monitor.http.ttfb")]
  #[sqlx(rename = "monitor.http.ttfb")]
  MonitorHttpTtfb,

  /// Http metric `monitor.http.transfer`
  ///
  /// Indicates how long it took the server to send the entire response.
  #[serde(rename = "monitor.http.transfer")]
  #[sqlx(rename = "monitor.http.transfer")]
  MonitorHttpTransfer,
}

/// Defines error codes describing the outcome of a monitor execution.
///
/// These codes are used to classify the known reason of an error.
/// [`ErrorType::None`] indicates a successful check, while
/// [`ErrorType::Unknown`] represents an unclassified error for which
/// detailed information should be preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "monitor_error_type")]
#[sqlx(rename_all = "snake_case")]
pub enum ErrorType {
  /// No error occurred during the check.
  None,

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

/// Represents a single measurement event produced by a monitor at
/// a concrete point in time.
#[derive(Debug)]
pub struct Metric {
  /// Unique identifier of the monitor that produced this metric.
  pub monitor_id: i64,

  /// Unix timestamp when the metric was created.
  pub timestamp: OffsetDateTime,

  /// The name of metric being reported.
  pub name: MetricName,

  /// Numeric value of the metric.
  pub value: f32,

  /// Type of error associated with this metric.
  pub error: MetricError,
}

/// Represents the presence or absence of an error during metric collection.
///
/// When `type` is [`ErrorType::None`], it indicates that no error occurred.
/// For complex errors that cannot be fully expressed by `ErrorType` alone,
/// additional details may be provided in `details` as a description.
#[derive(Debug)]
pub struct MetricError {
  /// Type of error by default is `ErrorType::None`
  pub r#type: ErrorType,

  /// Optional detailed description of the error.
  pub details: Option<String>,
}

impl MetricError {
  pub fn is_none(&self) -> bool {
    self.r#type == ErrorType::None
  }
}

impl From<&Measurement> for Vec<Metric> {
  fn from(measurement: &Measurement) -> Self {
    let make = |name: MetricName, value: f32| Metric {
      monitor_id: measurement.monitor_id,
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
      vec![make(MetricName::MonitorUp, 0f32)]
    } else {
      match measurement.data.as_ref() {
        Some(Data::Ping(data)) => vec![
          make(MetricName::MonitorUp, 1f32),
          make(MetricName::MonitorPingDns, data.dns),
          make(MetricName::MonitorPingRtt, data.rtt),
        ],
        Some(Data::Http(data)) => vec![
          make(MetricName::MonitorUp, 1f32),
          make(MetricName::MonitorHttpDns, data.dns),
          make(MetricName::MonitorHttpTcp, data.tcp),
          make(MetricName::MonitorHttpTls, data.tls),
          make(MetricName::MonitorHttpTtfb, data.ttfb),
          make(MetricName::MonitorHttpTransfer, data.transfer),
        ],
        None => vec![],
      }
    }
  }
}

impl Default for MetricError {
  fn default() -> Self {
    Self {
      r#type: ErrorType::None,
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
  use crate::models::monitor::measurement::{HttpData, PingData};

  assert_impl_all!(MetricName: Clone, Copy);
  assert_impl_all!(ErrorType: Clone, Copy);

  #[rstest]
  #[case(MetricName::MonitorUp, "monitor.up")]
  #[case(MetricName::MonitorSslDaysLeft, "monitor.ssl.days_left")]
  #[case(MetricName::MonitorDomainDaysLeft, "monitor.domain.days_left")]
  #[case(MetricName::MonitorPingDns, "monitor.ping.dns")]
  #[case(MetricName::MonitorPingRtt, "monitor.ping.rtt")]
  #[case(MetricName::MonitorHttpDns, "monitor.http.dns")]
  #[case(MetricName::MonitorHttpTcp, "monitor.http.tcp")]
  #[case(MetricName::MonitorHttpTls, "monitor.http.tls")]
  #[case(MetricName::MonitorHttpTtfb, "monitor.http.ttfb")]
  #[case(MetricName::MonitorHttpTransfer, "monitor.http.transfer")]
  fn test_metric_name_serialization(#[case] metric: MetricName, #[case] expected_str: &str) {
    let json = serde_json::to_string(&metric).unwrap();
    assert_eq!(json, format!("\"{}\"", expected_str));

    let parsed: MetricName = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, metric);
  }

  #[rstest]
  #[case(ErrorType::None, "none")]
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
    vec![
      (MetricName::MonitorUp, 1.0),
      (MetricName::MonitorPingDns, 10.0),
      (MetricName::MonitorPingRtt, 20.0),
    ]
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
    vec![
      (MetricName::MonitorUp, 1.0),
      (MetricName::MonitorHttpDns, 15.0),
      (MetricName::MonitorHttpTcp, 25.0),
      (MetricName::MonitorHttpTls, 35.0),
      (MetricName::MonitorHttpTtfb, 45.0),
      (MetricName::MonitorHttpTransfer, 55.0),
    ]
  )]
  #[case(
    Measurement {
      monitor_id: 3,
      timestamp: OffsetDateTime::now_utc(),
      data: None,
      error: Some(CollectorError::Ping(PingError::Timeout { timeout: 5 })),
    },
    vec![
    (MetricName::MonitorUp, 0f32)
    ]
  )]
  #[case(
    Measurement {
      monitor_id: 4,
      timestamp: OffsetDateTime::now_utc(),
      data: None,
      error: None,
    },
    vec![]
  )]
  fn test_measurement_to_metrics(
    #[case] measurement: Measurement,
    #[case] expected: Vec<(MetricName, f32)>,
  ) {
    let metrics: Vec<Metric> = (&measurement).into();

    assert_eq!(
      metrics.len(),
      expected.len(),
      "Unexpected number of metrics"
    );

    for (metric, (name, value)) in metrics.iter().zip(expected) {
      assert_eq!(metric.name, name);
      assert_eq!(metric.value, value);
    }
  }

  #[test]
  fn test_metric_error_is_none() {
    assert_eq!(MetricError::default().is_none(), true);
    assert_eq!(
      MetricError {
        r#type: ErrorType::Unknown,
        details: None
      }
      .is_none(),
      false
    );
  }

  #[test]
  fn test_metric_error_default() {
    let value = MetricError::default();

    assert_eq!(value.r#type, ErrorType::None);
    assert!(value.details.is_none());
  }
}
