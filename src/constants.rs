/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

/// The base URL for the Tradier REST API.
/// This URL is used for making standard HTTP requests to Tradier's API endpoints.
pub(crate) const TRADIER_API_BASE_URL: &str = "https://api.tradier.com";

/// The base WebSocket URL for the Tradier streaming API.
/// This URL is used for establishing WebSocket connections to stream real-time market data.
pub(crate) const TRADIER_WS_BASE_URL: &str = "wss://ws.tradier.com";

/// The base HTTP streaming URL for the Tradier streaming API.
/// This URL is used for making HTTP requests to Tradier's streaming endpoints for real-time data.
pub(crate) const TRADIER_STREAM_HTTP_BASE_URL: &str = "https://stream.tradier.com";

/// The events path for accessing market events in the Tradier streaming API.
/// This path is appended to the base URL (either WebSocket or HTTP) to access market event streams.
pub(crate) const TRADIER_STREAM_EVENTS_PATH: &str = "/v1/markets/events";

/// The default session timeout in seconds for Tradier API sessions.
/// This value is used to set timeout limits for API session-based requests.
pub(crate) const TRADIER_SESSION_TIMEOUT: i64 = 5;
