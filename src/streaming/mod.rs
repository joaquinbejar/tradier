//! # Tradier streaming
//!
//! Building blocks for the REST session bootstrap and the HTTP
//! chunked-transfer event streams.
//!
//! - [`http_stream`] exposes the HTTP-stream helpers —
//!   `Stream`-returning functions that reuse the pooled
//!   `reqwest::Client` on a
//!   [`crate::client::non_blocking::TradierRestClient`]. These are the
//!   HTTP fallback when WebSockets are not reachable.
//!
//! The REST endpoint that mints the session id itself lives under
//! [`crate::wssession::session`] (it is shared between the WebSocket
//! and HTTP streaming flavors).

pub mod http_stream;
