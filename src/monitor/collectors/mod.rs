mod http;
#[cfg(not(tarpaulin_include))]
// Excluded from coverage since ping requires raw sockets and elevated privileges.
mod ping;

pub use http::Http;
pub use ping::Ping;
