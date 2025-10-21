use crate::constants::{
    TRADIER_API_BASE_URL, TRADIER_STREAM_EVENTS_PATH, TRADIER_STREAM_HTTP_BASE_URL,
    TRADIER_WS_BASE_URL,
};
use serde::Deserialize;
use std::env;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use tracing::error;

/// The `Credentials` struct stores sensitive information required for
/// authenticating with the Tradier API, including client ID and secret.
///
/// Fields:
/// - `client_id`: The client ID for Tradier API authentication.
/// - `client_secret`: The client secret for Tradier API authentication.
/// - `access_token`: An optional access token for session-based authentication.
/// - `refresh_token`: An optional refresh token for renewing the access token.
#[derive(Debug, Deserialize, Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

/// The `Config` struct encapsulates the main configuration for the Tradier API library,
/// including credentials, REST API settings, and WebSocket streaming settings.
///
/// Fields:
/// - `credentials`: Holds API credentials for authentication.
/// - `rest_api`: Configuration for REST API interactions, including URL and timeout.
/// - `streaming`: Configuration for streaming interactions, including HTTP and WS URLs and settings.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub credentials: Credentials,
    pub rest_api: RestApiConfig,
    pub streaming: StreamingConfig,
}

/// The `RestApiConfig` struct holds configuration specific to REST API interactions.
///
/// Fields:
/// - `base_url`: The base URL for the Tradier REST API.
/// - `timeout`: The timeout duration (in seconds) for REST API requests.
#[derive(Debug, Deserialize, Clone)]
pub struct RestApiConfig {
    pub base_url: String,
    pub timeout: u64,
}

/// The `StreamingConfig` struct holds configuration specific to streaming interactions via HTTP or WebSocket.
///
/// Fields:
/// - `http_base_url`: The base URL for HTTP streaming.
/// - `ws_base_url`: The base URL for WebSocket streaming.
/// - `events_path`: Path for event streams.
/// - `reconnect_interval`: Interval (in seconds) for reconnect attempts.
#[derive(Debug, Deserialize, Clone)]
pub struct StreamingConfig {
    pub http_base_url: String,
    pub ws_base_url: String,
    pub events_path: String,
    pub reconnect_interval: u64,
}

/// Implements `fmt::Display` for `Credentials`, providing a JSON-style output
/// with redacted sensitive information for security.
impl fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\"client_id\":\"[REDACTED]\",\"client_secret\":\"[REDACTED]\",\"access_token\":{},\"refresh_token\":{}}}",
               self.access_token.as_ref().map_or("null".to_string(), |_| "\"[REDACTED]\"".to_string()),
               self.refresh_token.as_ref().map_or("null".to_string(), |_| "\"[REDACTED]\"".to_string()))
    }
}

/// Implements `fmt::Display` for `Config`, displaying all fields in JSON format.
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"credentials\":{},\"rest_api\":{},\"streaming\":{}}}",
            self.credentials, self.rest_api, self.streaming
        )
    }
}

/// Implements `fmt::Display` for `RestApiConfig`, displaying the REST API base URL and timeout in JSON format.
impl fmt::Display for RestApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"base_url\":\"{}\",\"timeout\":{}}}",
            self.base_url, self.timeout
        )
    }
}

/// Implements `fmt::Display` for `StreamingConfig`, displaying HTTP and WebSocket URLs,
/// event path, and reconnect interval in JSON format.
impl fmt::Display for StreamingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"http_base_url\":\"{}\",\"ws_base_url\":\"{}\",\"events_path\":\"{}\",\"reconnect_interval\":{}}}",
            self.http_base_url, self.ws_base_url, self.events_path, self.reconnect_interval
        )
    }
}

/// Retrieves an environment variable or returns a default value if the variable is missing or cannot be parsed.
///
/// Parameters:
/// - `env_var`: The name of the environment variable.
/// - `default`: The default value to return if the environment variable is not set or unparsable.
///
/// Returns:
/// - The value of the environment variable if available and valid; otherwise, the default value.
pub fn get_env_or_default<T: FromStr>(env_var: &str, default: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    match env::var(env_var) {
        Ok(val) => val.parse::<T>().unwrap_or_else(|_| {
            error!("Failed to parse {}: {}, using default", env_var, val);
            default
        }),
        Err(_) => default,
    }
}

impl Default for Config {
    /// Creates a default `Config` instance by calling `Config::new`.
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Constructs a new `Config` instance with environment variables or defaults if unavailable.
    ///
    /// Returns:
    /// - `Config` instance initialized with credentials, REST API, and streaming settings.
    pub fn new() -> Self {
        Config {
            credentials: Credentials {
                client_id: get_env_or_default(
                    "TRADIER_CLIENT_ID",
                    String::from("default_client_id"),
                ),
                client_secret: get_env_or_default(
                    "TRADIER_CLIENT_SECRET",
                    String::from("default_client_secret"),
                ),
                access_token: env::var("TRADIER_ACCESS_TOKEN").ok(),
                refresh_token: env::var("TRADIER_REFRESH_TOKEN").ok(),
            },
            rest_api: RestApiConfig {
                base_url: get_env_or_default(
                    "TRADIER_REST_BASE_URL",
                    String::from(TRADIER_API_BASE_URL),
                ),
                timeout: get_env_or_default("TRADIER_REST_TIMEOUT", 30),
            },
            streaming: StreamingConfig {
                http_base_url: get_env_or_default(
                    "TRADIER_STREAM_HTTP_BASE_URL",
                    String::from(TRADIER_STREAM_HTTP_BASE_URL),
                ),
                ws_base_url: get_env_or_default(
                    "TRADIER_WS_BASE_URL",
                    String::from(TRADIER_WS_BASE_URL),
                ),
                events_path: get_env_or_default(
                    "TRADIER_STREAM_EVENTS_PATH",
                    String::from(TRADIER_STREAM_EVENTS_PATH),
                ),
                reconnect_interval: get_env_or_default("TRADIER_STREAM_RECONNECT_INTERVAL", 5),
            },
        }
    }

    /// Generates the WebSocket URL by concatenating the base WebSocket URL and event path.
    ///
    /// Returns:
    /// - A `String` containing the full WebSocket URL.
    pub fn get_ws_url(&self) -> String {
        format!(
            "{}{}",
            self.streaming.ws_base_url, self.streaming.events_path
        )
    }

    /// Generates the HTTP streaming URL by concatenating the base HTTP URL and event path.
    ///
    /// Returns:
    /// - A `String` containing the full HTTP streaming URL.
    pub fn get_http_url(&self) -> String {
        format!(
            "{}{}",
            self.streaming.http_base_url, self.streaming.events_path
        )
    }
}

#[cfg(test)]
mod tests_config {
    use super::*;
    use crate::test_support::with_env_vars;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Initializes global test setup.
    fn setup() {
        INIT.call_once(|| {
            // Initialize any global test setup here
        });
    }

    #[test]
    fn test_config_new_with_defaults() {
        setup();
        with_env_vars(vec![], || {
            let config = Config::new();
            assert_eq!(config.rest_api.base_url, TRADIER_API_BASE_URL);
            assert_eq!(config.streaming.http_base_url, TRADIER_STREAM_HTTP_BASE_URL);
            assert_eq!(config.streaming.ws_base_url, TRADIER_WS_BASE_URL);
            assert_eq!(config.streaming.events_path, TRADIER_STREAM_EVENTS_PATH);
            assert_eq!(config.streaming.reconnect_interval, 5);
            assert_eq!(config.rest_api.timeout, 30);
        });
    }

    #[test]
    fn test_config_new_with_env_vars() {
        setup();
        with_env_vars(
            vec![
                ("TRADIER_CLIENT_ID", "test_client_id"),
                ("TRADIER_CLIENT_SECRET", "test_client_secret"),
                ("TRADIER_ACCESS_TOKEN", "test_access_token"),
                ("TRADIER_REFRESH_TOKEN", "test_refresh_token"),
                ("TRADIER_REST_BASE_URL", "https://test-api.tradier.com"),
                ("TRADIER_REST_TIMEOUT", "60"),
                (
                    "TRADIER_STREAM_HTTP_BASE_URL",
                    "https://test-stream.tradier.com",
                ),
                ("TRADIER_WS_BASE_URL", "wss://test-ws.tradier.com"),
                ("TRADIER_STREAM_EVENTS_PATH", "/v1/test/events"),
                ("TRADIER_STREAM_RECONNECT_INTERVAL", "10"),
            ],
            || {
                let config = Config::new();
                assert_eq!(config.credentials.client_id, "test_client_id");
                assert_eq!(config.credentials.client_secret, "test_client_secret");
                assert_eq!(
                    config.credentials.access_token,
                    Some("test_access_token".to_string())
                );
                assert_eq!(
                    config.credentials.refresh_token,
                    Some("test_refresh_token".to_string())
                );
                assert_eq!(config.rest_api.base_url, "https://test-api.tradier.com");
                assert_eq!(config.rest_api.timeout, 60);
                assert_eq!(
                    config.streaming.http_base_url,
                    "https://test-stream.tradier.com"
                );
                assert_eq!(config.streaming.ws_base_url, "wss://test-ws.tradier.com");
                assert_eq!(config.streaming.events_path, "/v1/test/events");
                assert_eq!(config.streaming.reconnect_interval, 10);
            },
        );
    }

    #[test]
    fn test_get_ws_url() {
        setup();
        with_env_vars(vec![], || {
            let config = Config::new();
            assert_eq!(
                config.get_ws_url(),
                format!("{}{}", TRADIER_WS_BASE_URL, TRADIER_STREAM_EVENTS_PATH)
            );
        });
    }

    #[test]
    fn test_get_http_url() {
        setup();
        with_env_vars(vec![], || {
            let config = Config::new();
            assert_eq!(
                config.get_http_url(),
                format!(
                    "{}{}",
                    TRADIER_STREAM_HTTP_BASE_URL, TRADIER_STREAM_EVENTS_PATH
                )
            );
        });
    }

    #[test]
    fn test_credentials_display() {
        setup();
        let credentials = Credentials {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            access_token: Some("test_access".to_string()),
            refresh_token: Some("test_refresh".to_string()),
        };
        let display = credentials.to_string();
        assert!(display.contains("\"client_id\":\"[REDACTED]\""));
        assert!(display.contains("\"client_secret\":\"[REDACTED]\""));
        assert!(display.contains("\"access_token\":\"[REDACTED]\""));
        assert!(display.contains("\"refresh_token\":\"[REDACTED]\""));
    }

    #[test]
    fn test_config_display() {
        setup();
        with_env_vars(vec![], || {
            let config = Config::new();
            let display = config.to_string();
            assert!(display.contains("\"credentials\":"));
            assert!(display.contains("\"rest_api\":"));
            assert!(display.contains("\"streaming\":"));
        });
    }
}
