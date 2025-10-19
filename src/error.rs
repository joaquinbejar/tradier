use reqwest::StatusCode;

use crate::wssession::session::SessionType;

/// A specialized `Result` type for the Tradier API client, using `Error` for errors.
pub type Result<T> = core::result::Result<T, Error>;

/// `Error` is a comprehensive error type for the Tradier API client, covering
/// issues that may arise during configuration, network requests, WebSocket connections,
/// and session management. It provides detailed error messages for common issues,
/// helping to identify the root cause of errors within the API client.
///
/// Variants:
/// - `UrlParsingError`: Occurs when a URL fails to parse, often due to an invalid URL format.
/// - `CreateSessionError`: Represents a failure in creating a session, providing the `SessionType`,
///   HTTP status, and response body for troubleshooting.
/// - `JsonParsingError`: Raised when parsing JSON data into the expected session response structure fails.
/// - `MissingAccessToken`: Indicates a missing access token, which is required for API authentication.
/// - `SessionAlreadyExists`: Raised when attempting to create a duplicate session where one already exists.
/// - `NetworkError`: Wraps network-related errors that occur during API requests, sourced from `reqwest`.
/// - `WebSocketError`: Wraps WebSocket-related errors, sourced from the `tungstenite` crate.
/// - `UnexpectedError`: Represents any other unexpected error with an accompanying error message.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error when parsing a URL fails.
    ///
    /// # Source
    /// This error wraps `url::ParseError`, allowing for more descriptive error reporting
    /// when invalid URL strings are encountered.
    #[error("URL Parsing Error: {0}")]
    UrlParsingError(#[from] url::ParseError),

    /// Error that occurs when creating a session fails, including session type, HTTP status, and response body.
    ///
    /// # Parameters
    /// - `SessionType`: Type of session (e.g., `Account`, `Market`) attempted to create.
    /// - `StatusCode`: HTTP status code returned by the API.
    /// - `String`: Body of the API response.
    #[error("Failed to create {0} session. Status: {1}. Body: {2}")]
    CreateSessionError(SessionType, StatusCode, String),

    /// Error if an unsupported MarketSessionFilter is encountered
    ///
    /// # Parameters
    /// - `MarketSessionFilter`: Type of Event Filter that was encountered.
    #[error("Unsupported MarketSessionFilter: {0}")]
    UnsupportedMarketFilter(String),

    /// Error when JSON parsing fails while handling session response data.
    ///
    /// # Source
    /// Wraps `serde_json::Error` to identify issues in converting API response data to the expected structure.
    #[error("Unable to parse Session Response")]
    JsonParsingError(#[from] serde_json::Error),

    /// Error when an access token, required for authentication, is missing.
    #[error("Missing Access Token")]
    MissingAccessToken,

    /// Error raised when attempting to create a session that already exists.
    #[error("Session already exists")]
    SessionAlreadyExists,

    /// Represents an IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents errors during network requests to the Tradier API.
    ///
    /// # Source
    /// Wraps `reqwest::Error` to capture issues such as connectivity problems, timeouts, or HTTP-level errors.
    #[error("Network error during API request")]
    NetworkError(#[from] reqwest::Error),

    /// Error raised during WebSocket operations.
    ///
    /// # Source
    /// Wraps `tungstenite::Error` for more detailed WebSocket connection error reporting.
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] Box<tungstenite::Error>),

    /// Represents any unexpected error, including a custom message for additional context.
    ///
    /// # Parameters
    /// - `String`: Custom message describing the unexpected error.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
