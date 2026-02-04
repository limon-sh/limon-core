//! A module containing a set of common models.

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

#[cfg(test)]
mod tests {
  use static_assertions::assert_impl_all;

  use super::*;

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
}
