//! Market Data REST endpoints.
//!
//! Provides typed bindings for the Tradier `markets/*` REST surface,
//! exposed both as blocking and non-blocking traits.
//!
//! Upstream docs: <https://documentation.tradier.com/brokerage-api/markets/>.

pub mod api;
#[cfg(test)]
pub(crate) mod test_support;
pub mod types;
