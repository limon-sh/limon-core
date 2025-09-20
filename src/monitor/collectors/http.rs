use std::time::Duration;

use curl::easy::{Easy2, Handler, HttpVersion, List, WriteError};
use tokio::task;

use crate::monitor::errors::HttpError;
use crate::monitor::models::{Data, HttpConfig, HttpData};

#[derive(Default)]
struct ResponseBody(Vec<u8>);

impl Handler for ResponseBody {
  fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
    self.0.extend_from_slice(data);

    Ok(data.len())
  }
}

impl ResponseBody {
  pub fn get_body(&self) -> String {
    String::from_utf8_lossy(&self.0).into()
  }
}

pub struct Http;

impl Http {
  pub async fn measure(host: &String, config: &HttpConfig) -> Result<Data, HttpError> {
    let url = format!(
      "{}://{}{}{}",
      config.protocol.to_lowercase(),
      host,
      config
        .port
        .map_or(String::new(), |port| format!(":{}", port)),
      config.path.clone().unwrap_or_default()
    );

    let mut headers = List::new();
    if let Some(header) = &config.header {
      headers.append(&format!("{}: {}", header.name, header.value))?;
    }

    let mut request = Easy2::new(ResponseBody::default());
    request.url(url.as_str())?;
    request.http_headers(headers)?;
    request.timeout(Duration::from_secs(config.timeout as u64))?;
    request.cookie_file("")?;
    request.follow_location(config.follow_redirects)?;
    request.http_version(HttpVersion::V2)?;

    match config.method.to_lowercase().as_str() {
      "get" => request.get(true)?,
      "post" => request.post(true)?,
      "put" => request.put(true)?,
      "patch" => request.custom_request("PATCH")?,
      "head" => {
        request.nobody(true)?;
        request.custom_request("HEAD")?
      }
      _ => unimplemented!("Unimplemented HTTP method"),
    };

    if let Some(body) = config.body.clone() {
      request.post_fields_copy(body.as_bytes())?;
    }

    let response = task::spawn_blocking(move || match request.perform() {
      Ok(()) => Ok(request),
      Err(error) => Err(HttpError::Unknown(error)),
    })
    .await
    .expect("curl request")?;

    let response_status = response.response_code()? as u16;
    let expected_status_code = config.expected_status_code as u16;

    if response_status != expected_status_code {
      return Err(HttpError::StatusMismatch {
        expected: expected_status_code,
        actual: response_status,
      });
    }

    if let Some(keyword) = config.keyword.clone() {
      let response_body = response.get_ref().get_body();

      if !response_body.contains(keyword.as_str()) {
        return Err(HttpError::KeywordNotFound { keyword });
      }
    }

    Ok(Data::Http(HttpData {
      dns_lookup: response.namelookup_time()?.as_secs_f32(),
      connect: response.connect_time()?.as_secs_f32(),
      tls_handshake: response.appconnect_time()?.as_secs_f32(),
      data_transfer: (response.total_time()? - response.starttransfer_time()?).as_secs_f32(),
    }))
  }
}

#[cfg(test)]
mod tests {
  use httpmock::prelude::*;

  use super::*;
  use crate::monitor::models::Header;

  #[test]
  fn response_body() {
    let mut response_body = ResponseBody([0].into());

    assert!(
      response_body.write(&[0]).is_ok(),
      "response body is writable"
    );
    assert_eq!(
      response_body.get_body(),
      "\0\0",
      "response body is readable"
    );
  }

  #[tokio::test]
  async fn headers() {
    let server = MockServer::start_async().await;

    let mock = server
      .mock_async(|when, then| {
        when
          .header("Authorization", "token")
          .method(GET)
          .path("/check");
        then.status(200);
      })
      .await;

    let result = Http::measure(&server.host(), &HttpConfig {
      timeout: 3,
      method: String::from("GET"),
      protocol: String::from("HTTP"),
      port: Some(server.port()),
      path: Some(String::from("/check")),
      header: Some(Header {
        name: String::from("Authorization"),
        value: String::from("token"),
      }),
      expected_status_code: 200,
      ..Default::default()
    })
    .await;

    mock.assert();

    assert!(result.is_ok(), "request header is correct");
  }

  #[tokio::test]
  async fn body() {
    let server = MockServer::start_async().await;

    let mock = server
      .mock_async(|when, then| {
        when.method(POST).path("/check").body("test");
        then.status(200);
      })
      .await;

    let result = Http::measure(&server.host(), &HttpConfig {
      timeout: 3,
      method: String::from("POST"),
      protocol: String::from("HTTP"),
      port: Some(server.port()),
      path: Some(String::from("/check")),
      body: Some(String::from("test")),
      expected_status_code: 200,
      ..Default::default()
    })
    .await;

    mock.assert();

    assert!(result.is_ok(), "request body is correct");
  }

  #[tokio::test]
  async fn methods() {
    let server = MockServer::start_async().await;

    for method in ["GET", "POST", "PUT", "PATCH", "HEAD"] {
      let mock = server
        .mock_async(|when, then| {
          when.method(Method::from(method)).path("/check");
          then.status(200);
        })
        .await;

      let result = Http::measure(&server.host(), &HttpConfig {
        timeout: 3,
        method: String::from(method),
        protocol: String::from("HTTP"),
        port: Some(server.port()),
        path: Some(String::from("/check")),
        expected_status_code: 200,
        ..Default::default()
      })
      .await;

      mock.assert();

      assert!(result.is_ok(), "request method is correct");
    }
  }

  #[tokio::test]
  async fn response_status_mismatch() {
    let server = MockServer::start_async().await;

    let mock = server
      .mock_async(|when, then| {
        when.method(GET).path("/check");
        then.status(400);
      })
      .await;

    let result = Http::measure(&server.host(), &HttpConfig {
      timeout: 3,
      method: String::from("GET"),
      protocol: String::from("HTTP"),
      port: Some(server.port()),
      path: Some(String::from("/check")),
      expected_status_code: 200,
      ..Default::default()
    })
    .await;

    mock.assert();

    assert!(result.is_err(), "response has unexpected status");
  }

  #[tokio::test]
  async fn response_doesnt_contain_keyword() {
    let server = MockServer::start_async().await;

    let mock = server
      .mock_async(|when, then| {
        when.method(GET).path("/check");
        then.status(200).body("error");
      })
      .await;

    let result = Http::measure(&server.host(), &HttpConfig {
      timeout: 3,
      method: String::from("GET"),
      protocol: String::from("HTTP"),
      port: Some(server.port()),
      path: Some(String::from("/check")),
      expected_status_code: 200,
      keyword: Some(String::from("index")),
      ..Default::default()
    })
    .await;

    mock.assert();

    assert!(result.is_err(), "response doesn't contain expected keyword");
  }

  #[tokio::test]
  async fn unknown_error() {
    let result = Http::measure(&String::from("127.0.0.1"), &HttpConfig {
      method: String::from("GET"),
      protocol: String::from("HTTP"),
      port: Some(5555),
      expected_status_code: 200,
      ..Default::default()
    })
    .await;

    assert!(result.is_err(), "Could not connect to server");
  }
}
