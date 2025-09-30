use crate::schedule::Schedulable;

/// Represents a monitor for a host, which can be measured.
#[derive(Debug)]
pub struct Monitor {
  /// Monitor identifier.
  pub id: i64,

  /// Host without protocol specified.
  pub host: String,

  /// Monitor's config.
  pub config: Config,
}

/// Configuration type for a monitor.
#[derive(Debug)]
pub enum Config {
  /// Ping monitor configuration.
  Ping(PingConfig),

  /// HTTP monitor configuration.
  Http(HttpConfig),
}

/// Configuration for a Ping monitor.
#[derive(Debug, Default, serde::Deserialize)]
pub struct PingConfig {
  /// How often the monitor should perform a check, in seconds.
  pub check_frequency: i64,

  /// Number of consecutive successful checks required to confirm a state change.
  pub confirmation_period: i64,

  /// Number of consecutive failed checks required to consider the monitor recovered.
  pub recovery_period: i64,

  /// Maximum time, in seconds, to wait for a ping response before timing out.
  pub timeout: i64,
}

/// Configuration for an `HTTP` monitor.
#[derive(Debug, Default, serde::Deserialize)]
pub struct HttpConfig {
  /// How often the monitor should perform a check, in seconds.
  pub check_frequency: i64,

  /// Number of consecutive successful checks required to confirm a state change.
  pub confirmation_period: i64,

  /// Number of consecutive failed checks required to consider the monitor recovered.
  pub recovery_period: i64,

  /// Maximum time, in seconds, to wait for an `HTTP` response before timing out.
  pub timeout: i32,

  /// HTTP method to use (e.g., `GET`, `POST`).
  pub method: String,

  /// Protocol to use (`HTTP` or `HTTPS`).
  pub protocol: String,

  /// Optional port number. If `None`, defaults to 80 for `HTTP` and 443 for `HTTPS`.
  pub port: Option<u16>,

  /// Optional request path (e.g., "/health").
  pub path: Option<String>,

  /// Optional request body for methods like `POST` or `PUT`.
  pub body: Option<String>,

  /// Optional keyword to search for in the response body.
  pub keyword: Option<String>,

  /// Expected `HTTP` status code.
  pub expected_status_code: i32,

  /// Whether to follow `HTTP` redirects.
  pub follow_redirects: bool,

  /// Whether to keep cookies when following redirects.
  pub keep_cookies_on_redirects: bool,

  /// Optional `HTTP` headers to include in the request.
  pub header: Option<Header>,
}

/// Represents a single `HTTP` header (name-value pair).
#[derive(Debug, serde::Deserialize)]
pub struct Header {
  /// The name of the `HTTP` header (e.g., `"Content-Type"`).
  pub name: String,

  /// The value of the `HTTP` header (e.g., `"application/json"`).
  pub value: String,
}

/// Trait implementation for scheduling monitors.
impl Schedulable for Monitor {
  type Id = i64;
  type Interval = i64;

  fn get_id(&self) -> Self::Id {
    self.id
  }

  fn get_interval(&self) -> Self::Interval {
    match &self.config {
      Config::Ping(config) => config.check_frequency,
      Config::Http(config) => config.check_frequency,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn monitor_ping_is_schedulable() {
    let monitor = Monitor {
      id: 1,
      host: String::from("test"),
      config: Config::Ping(PingConfig {
        check_frequency: 10,
        ..Default::default()
      }),
    };

    assert_eq!(monitor.get_id(), 1, "monitor id is correct");
    assert_eq!(monitor.get_interval(), 10, "monitor interval is correct");
  }

  #[test]
  fn monitor_http_is_schedulable() {
    let monitor = Monitor {
      id: 1,
      host: String::from("test"),
      config: Config::Http(HttpConfig {
        check_frequency: 10,
        ..Default::default()
      }),
    };

    assert_eq!(monitor.get_id(), 1, "monitor id is correct");
    assert_eq!(monitor.get_interval(), 10, "monitor interval is correct");
  }
}
