//! Fundamentals (beta) REST endpoints.
//!
//! Provides typed bindings for the Tradier `beta/markets/fundamentals/*`
//! REST surface, exposed both as blocking and non-blocking traits.
//!
//! Upstream docs: <https://documentation.tradier.com/brokerage-api/markets/get-company>.

#[cfg(test)]
pub(crate) mod test_support;
pub mod types;
