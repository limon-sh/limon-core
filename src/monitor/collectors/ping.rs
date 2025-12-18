use tokio::time::{Duration, timeout};

use crate::monitor::collectors::dns::Dns;
use crate::monitor::errors::PingError;
use crate::monitor::models::{Data, PingConfig, PingData};

pub struct Ping;

impl Ping {
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
    .map_err(|error| PingError::Error(error))?;

    Ok(Data::Ping(PingData {
      dns_lookup: lookup_duration.as_secs_f32(),
      ping: ping_duration.as_secs_f32(),
    }))
  }
}
