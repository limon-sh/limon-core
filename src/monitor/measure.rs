use time::OffsetDateTime;

use crate::monitor::collectors::{Http, Ping};
use crate::monitor::errors::CollectorError;
use crate::monitor::models::{Config, Data, Measurement, Monitor};

#[doc(hidden)]
#[macro_export]
macro_rules! measure {
  ($block:block) => {{
    let start = std::time::Instant::now();
    let result = { $block };

    (result, start.elapsed())
  }};
}

impl Monitor {
  /// Performs a measurement for this monitor asynchronously.
  ///
  /// The exact behavior depends on the type of configuration (`self.config`):
  ///
  /// - **`Config::Ping`** – Sends a network ping to the monitor's host using
  ///   the settings in the Ping configuration.
  /// - **`Config::Http`** – Performs an HTTP request to the monitor's host
  ///   using the parameters in [`HttpConfig`](crate::monitor::models::HttpConfig),
  ///   such as method, path, timeout, expected status code, and follow redirects.
  ///
  /// The returned [`Measurement`] includes:
  /// - [`data`](Measurement#structfield.data): containing the collected
  ///   measurement if successful.
  /// - [`error`](Measurement#structfield.error): containing any error
  ///   that occurred during the measurement.
  pub async fn measure(&self) -> Measurement {
    let mut measure = Measurement {
      timestamp: OffsetDateTime::now_utc(),
      monitor_id: self.id,
      data: None,
      error: None,
    };

    let result: Result<Data, CollectorError> = match &self.config {
      #[cfg(not(tarpaulin_include))]
      // This branch is excluded from code coverage (`tarpaulin_include`) because
      // raw sockets are required for performing ICMP (ping) measurements.
      // Such operations usually cannot be executed in test environments, since
      // they require elevated privileges or special OS-level capabilities.
      Config::Ping(config) => Ping::measure(&self.host, config)
        .await
        .map_err(|error| error.into()),
      Config::Http(config) => Http::measure(&self.host, config)
        .await
        .map_err(|error| error.into()),
    };

    if result.is_ok() {
      measure.data = result.ok();
    } else {
      measure.error = result.err();
    }

    measure
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use httpmock::Method::GET;
  use httpmock::MockServer;

  use super::*;
  use crate::monitor::models::{Header, HttpConfig};

  #[test]
  fn measure_macro() {
    let ((), elapsed) = measure!({
      std::thread::sleep(Duration::from_millis(50));
    });

    assert!(elapsed >= Duration::from_millis(50));
  }

  #[tokio::test]
  async fn measure_http_with_data() {
    let server = MockServer::start_async().await;

    let mock = server
      .mock_async(|when, then| {
        when
          .header("Authorization", "token")
          .method(GET)
          .path("/check");
        then.status(200).body("index");
      })
      .await;

    let monitor = Monitor {
      id: 1,
      host: format!("{}:{}", &server.host(), &server.port()),
      config: Config::Http(HttpConfig {
        timeout: 3,
        method: String::from("GET"),
        protocol: String::from("HTTP"),
        path: Some(String::from("/check")),
        header: Some(Header {
          name: String::from("Authorization"),
          value: String::from("token"),
        }),
        expected_status_code: 200,
        keyword: Some(String::from("index")),
        ..Default::default()
      }),
    };

    let result = monitor.measure().await;

    mock.assert();

    assert!(
      result.data.is_some() && result.error.is_none(),
      "monitor measurement has data"
    );
  }

  #[tokio::test]
  async fn measure_http_with_error() {
    let server = MockServer::start_async().await;

    let mock = server
      .mock_async(|when, then| {
        when.method(GET).path("/check");
        then.status(400);
      })
      .await;

    let monitor = Monitor {
      id: 1,
      host: format!("{}:{}", &server.host(), &server.port()),
      config: Config::Http(HttpConfig {
        timeout: 3,
        method: String::from("GET"),
        protocol: String::from("HTTP"),
        path: Some(String::from("/check")),
        expected_status_code: 200,
        ..Default::default()
      }),
    };

    let result = monitor.measure().await;

    mock.assert();

    assert!(
      result.data.is_none() && result.error.is_some(),
      "monitor measurement has error"
    );
  }
}
