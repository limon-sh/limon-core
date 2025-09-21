use std::sync::Arc;
use std::time::Duration;

use fastping_rs::{PingResult, Pinger};
use once_cell::sync::Lazy;
use tokio::task;
use trust_dns_resolver::{TokioAsyncResolver, config::ResolverOpts, error::ResolveError};

use crate::measure;
use crate::monitor::errors::PingError;
use crate::monitor::models::{Data, PingConfig, PingData};

static RESOLVER: Lazy<Arc<TokioAsyncResolver>> = Lazy::new(|| {
  let mut opts = ResolverOpts::default();
  opts.cache_size = 0;
  opts.positive_min_ttl = Some(Duration::ZERO);
  opts.positive_max_ttl = Some(Duration::ZERO);
  opts.negative_min_ttl = Some(Duration::ZERO);
  opts.negative_max_ttl = Some(Duration::ZERO);

  Arc::new(TokioAsyncResolver::tokio_from_system_conf().expect("system resolver"))
});

pub struct Ping;

impl Ping {
  pub async fn measure(host: &String, config: &PingConfig) -> Result<Data, PingError> {
    let (lookup, lookup_duration) = measure!({ Arc::clone(&RESOLVER).lookup_ip(host).await? });
    let rtt = (config.timeout as u64).checked_mul(1000);
    let ip_address = lookup
      .iter()
      .next()
      .ok_or(ResolveError::from("No records found"))?;

    task::spawn_blocking(move || {
      let (pinger, results) = Pinger::new(rtt, Some(1000)).unwrap();
      pinger.add_ipaddr(&ip_address.to_string().as_str());
      pinger.run_pinger();

      match results.recv() {
        Ok(PingResult::Receive { addr: _, rtt }) => Ok(Data::Ping(PingData {
          dns_lookup: lookup_duration.as_secs_f32(),
          ping: rtt.as_secs_f32(),
        })),
        Ok(PingResult::Idle { addr }) => Err(PingError::NoReply {
          addr: addr.to_string(),
        }),
        Err(_) => Err(PingError::Unreachable),
      }
    })
    .await
    .expect("ping request")
  }
}
