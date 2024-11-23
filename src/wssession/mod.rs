//! # WebSocket Streaming Sessions
//!
//! This module provides interfaces for interacting with Tradier's streaming APIs, enabling real-time
//! data access for both account and market events. Tradier offers HTTP and WebSocket streaming APIs,
//! allowing clients to receive immediate updates as events occur. These APIs process and transmit
//! market data and account events in real-time to connected clients. For more information, refer to
//! the official [Tradier Streaming API documentation](https://documentation.tradier.com/brokerage-api/overview/streaming).
//!
//! ## Overview
//!
//! - **`AccountSession`**: Manages WebSocket sessions for streaming account-related events, such as
//!   order status updates and balance changes.
//! - **`MarketSession`**: Handles WebSocket sessions for streaming market data, including real-time
//!   quotes and trades.
//! - **`SessionManager`**: Ensures that only one streaming session is active at any given time, adhering
//!   to Tradier's limitation of a single concurrent session per user.
//!
//! ## Usage
//!
//! By utilizing these components, developers can integrate Tradier's streaming capabilities into
//! their applications, facilitating real-time data processing and event handling.

mod account;

mod market;

pub(crate) mod session;
pub(crate) mod session_manager;

pub use account::AccountSession;
pub use market::{MarketSession, MarketSessionFilter, MarketSessionPayload};
