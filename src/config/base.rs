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

#[derive(Debug, Deserialize, Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub credentials: Credentials,
    pub rest_api: RestApiConfig,
    pub streaming: StreamingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RestApiConfig {
    pub base_url: String,
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamingConfig {
    pub http_base_url: String,
    pub ws_base_url: String,
    pub events_path: String,
    pub reconnect_interval: u64,
}

impl fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\"client_id\":\"[REDACTED]\",\"client_secret\":\"[REDACTED]\",\"access_token\":{},\"refresh_token\":{}}}",
               self.access_token.as_ref().map_or("null".to_string(), |_| "\"[REDACTED]\"".to_string()),
               self.refresh_token.as_ref().map_or("null".to_string(), |_| "\"[REDACTED]\"".to_string()))
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"credentials\":{},\"rest_api\":{},\"streaming\":{}}}",
            self.credentials, self.rest_api, self.streaming
        )
    }
}

impl fmt::Display for RestApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"base_url\":\"{}\",\"timeout\":{}}}",
            self.base_url, self.timeout
        )
    }
}

impl fmt::Display for StreamingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"http_base_url\":\"{}\",\"ws_base_url\":\"{}\",\"events_path\":\"{}\",\"reconnect_interval\":{}}}",
            self.http_base_url, self.ws_base_url, self.events_path, self.reconnect_interval
        )
    }
}

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
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
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

    pub fn get_ws_url(&self) -> String {
        format!(
            "{}{}",
            self.streaming.ws_base_url, self.streaming.events_path
        )
    }

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
    use std::sync::Mutex;
    use std::sync::Once;

    static INIT: Once = Once::new();
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup() {
        INIT.call_once(|| {
            // Initialize any global test setup here
        });
    }

    fn with_env_vars<F>(vars: Vec<(&str, &str)>, test: F)
    where
        F: FnOnce(),
    {
        let _lock = ENV_MUTEX.lock().unwrap();
        let mut old_vars = Vec::new();

        for (key, value) in vars {
            old_vars.push((key, env::var(key).ok()));
            env::set_var(key, value);
        }

        test();

        for (key, value) in old_vars {
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
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
