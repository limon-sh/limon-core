use tokio::time::{Duration, timeout};

use crate::collectors::monitor::dns::Dns;
use crate::collectors::monitor::errors::PingError;
use crate::models::monitor::config::PingConfig;
use crate::models::monitor::measurement::{Data, PingData};

/// Collector for receiving monitor data from `ICMP` ping.
pub struct Ping;

impl Ping {
  #[cfg(not(tarpaulin_include))]
  pub async fn measure(host: &String, config: &PingConfig) -> Result<Data, PingError> {
    let (ip_addr, lookup_duration) = Dns::measure(host).await?;
    let (_, ping_duration) = timeout(
      Duration::from_secs(config.timeout as u64),
      surge_ping::ping(ip_addr, &[0; 8]),
    )
    .await
    .map_err(|_| PingError::Timeout {
      timeout: config.timeout,
    })?
    .map_err(|error| PingError::Unknown(error))?;

    Ok(Data::Ping(PingData {
      dns: lookup_duration.as_secs_f32(),
      rtt: ping_duration.as_secs_f32(),
    }))
  }
}
