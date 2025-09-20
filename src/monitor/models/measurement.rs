use crate::monitor::errors::CollectorError;

/// Represents a single measurement performed by a monitor.
///
/// Each `Measurement` records the timestamp of the check, the ID of the monitor,
/// and either the collected data or an error if the measurement failed.
#[derive(Debug)]
pub struct Measurement {
  /// Unix timestamp when the measurement was taken.
  pub timestamp: i64,

  /// Unique identifier of the monitor that produced this measurement.
  pub monitor_id: i64,

  /// Measurement data, if the operation was successful.
  pub data: Option<Data>,

  /// Error that occurred during the measurement.
  pub error: Option<CollectorError>,
}

/// The collected data of a measurement, which can be either a ping or HTTP measurement.
#[derive(Debug)]
pub enum Data {
  /// Data collected from a ping monitor.
  Ping(PingData),

  /// Data collected from an HTTP monitor.
  Http(HttpData),
}

/// Data returned by a ping monitor.
///
/// Contains timing information for DNS lookup and ICMP ping.
#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub struct PingData {
  /// Time in milliseconds spent on DNS resolution.
  pub dns_lookup: f32,

  /// Time in milliseconds spent performing the ping.
  pub ping: f32,
}

/// Data returned by an HTTP monitor.
///
/// Contains timing information for DNS resolution, TCP connection, TLS handshake,
/// and data transfer.
#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub struct HttpData {
  /// Time in milliseconds spent on DNS resolution.
  pub dns_lookup: f32,

  /// Time in milliseconds spent establishing the TCP connection.
  pub connect: f32,

  /// Time in milliseconds spent performing the TLS handshake
  pub tls_handshake: f32,

  /// Time in milliseconds spent transferring the HTTP response body.
  pub data_transfer: f32,
}
