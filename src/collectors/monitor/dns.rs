use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::error::ResolveError;

#[doc(hidden)]
#[macro_export]
macro_rules! measure {
  ($block:block) => {{
    let start = std::time::Instant::now();
    let result = { $block };

    (result, start.elapsed())
  }};
}

static _RESOLVER: Lazy<Arc<TokioAsyncResolver>> =
  Lazy::new(|| Arc::new(TokioAsyncResolver::tokio_from_system_conf().expect("system resolver")));

pub struct Dns;

impl Dns {
  pub async fn measure(host: &String) -> Result<(IpAddr, Duration), ResolveError> {
    let host = host.rsplit_once(':').map(|(h, _port)| h).unwrap_or(host);
    let (lookup_ip, lookup_duration) = measure!({ Arc::clone(&_RESOLVER).lookup_ip(host).await? });

    Ok((
      lookup_ip
        .iter()
        .next()
        .ok_or(ResolveError::from("No records found"))?,
      lookup_duration,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn resolve_localhost() {
    let (ip_addr, lookup_duration) = Dns::measure(&String::from("localhost")).await.unwrap();

    assert!(ip_addr.is_loopback());
    assert!(!lookup_duration.is_zero());
  }

  #[tokio::test]
  async fn resolve_localhost_with_port() {
    let (ip_addr, lookup_duration) = Dns::measure(&String::from("localhost:5555")).await.unwrap();

    assert!(ip_addr.is_loopback());
    assert!(!lookup_duration.is_zero());
  }
}
