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
    pub http_url: String,
    pub websocket_url: String,
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
            "{{\"http_url\":\"{}\",\"websocket_url\":\"{}\",\"reconnect_interval\":{}}}",
            self.http_url, self.websocket_url, self.reconnect_interval
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
                client_id: get_env_or_default("TRADIER_CLIENT_ID", String::from("default_client_id")),
                client_secret: get_env_or_default("TRADIER_CLIENT_SECRET", String::from("default_client_secret")),
                access_token: None,
                refresh_token: None,
            },
            rest_api: RestApiConfig {
                base_url: get_env_or_default(
                    "TRADIER_REST_BASE_URL",
                    String::from("https://api.tradier.com"),
                ),
                timeout: get_env_or_default("TRADIER_REST_TIMEOUT", 30),
            },
            streaming: StreamingConfig {
                http_url: get_env_or_default(
                    "TRADIER_STREAM_HTTP_URL",
                    String::from("https://stream.tradier.com/v1/markets/events"),
                ),
                websocket_url: get_env_or_default(
                    "TRADIER_STREAM_WS_URL",
                    String::from("wss://ws.tradier.com/v1/markets/events"),
                ),
                reconnect_interval: get_env_or_default("TRADIER_STREAM_RECONNECT_INTERVAL", 5),
            },
        }
    }
}

#[cfg(test)]
mod tests_config {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
    fn test_config_new_with_env_vars() {
        with_env_vars(
            vec![
                ("TRADIER_CLIENT_ID", "test_client_id"),
                ("TRADIER_CLIENT_SECRET", "test_client_secret"),
                ("TRADIER_REST_BASE_URL", "https://test-api.tradier.com"),
                ("TRADIER_REST_TIMEOUT", "60"),
                ("TRADIER_STREAM_HTTP_URL", "https://test-stream.tradier.com"),
                ("TRADIER_STREAM_WS_URL", "wss://test-ws.tradier.com"),
                ("TRADIER_STREAM_RECONNECT_INTERVAL", "10"),
            ],
            || {
                let config = Config::new();

                assert_eq!(config.credentials.client_id, "test_client_id");
                assert_eq!(config.credentials.client_secret, "test_client_secret");
                assert_eq!(config.rest_api.base_url, "https://test-api.tradier.com");
                assert_eq!(config.rest_api.timeout, 60);
                assert_eq!(config.streaming.http_url, "https://test-stream.tradier.com");
                assert_eq!(config.streaming.websocket_url, "wss://test-ws.tradier.com");
                assert_eq!(config.streaming.reconnect_interval, 10);
            },
        );
    }

    #[test]
    fn test_config_new_with_default_values() {
        with_env_vars(vec![], || {
            let config = Config::new();

            assert_eq!(config.credentials.client_id, "default_client_id");
            assert_eq!(config.credentials.client_secret, "default_client_secret");
            assert_eq!(config.rest_api.base_url, "https://api.tradier.com");
            assert_eq!(config.rest_api.timeout, 30);
            assert_eq!(config.streaming.http_url, "https://stream.tradier.com/v1/markets/events");
            assert_eq!(config.streaming.websocket_url, "wss://ws.tradier.com/v1/markets/events");
            assert_eq!(config.streaming.reconnect_interval, 5);
        });
    }

    #[test]
    fn test_credentials_display() {
        let credentials = Credentials {
            client_id: "client123".to_string(),
            client_secret: "secret123".to_string(),
            access_token: Some("access_token123".to_string()),
            refresh_token: None,
        };

        let display_output = credentials.to_string();
        let expected_json = json!({
            "client_id": "[REDACTED]",
            "client_secret": "[REDACTED]",
            "access_token": "[REDACTED]",
            "refresh_token": null
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_rest_api_config_display() {
        let rest_api_config = RestApiConfig {
            base_url: "https://api.tradier.com".to_string(),
            timeout: 30,
        };

        let display_output = rest_api_config.to_string();
        let expected_json = json!({
            "base_url": "https://api.tradier.com",
            "timeout": 30
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_streaming_config_display() {
        let streaming_config = StreamingConfig {
            http_url: "https://stream.tradier.com/v1/markets/events".to_string(),
            websocket_url: "wss://ws.tradier.com/v1/markets/events".to_string(),
            reconnect_interval: 5,
        };

        let display_output = streaming_config.to_string();
        let expected_json = json!({
            "http_url": "https://stream.tradier.com/v1/markets/events",
            "websocket_url": "wss://ws.tradier.com/v1/markets/events",
            "reconnect_interval": 5
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_config_display() {
        let config = Config {
            credentials: Credentials {
                client_id: "client123".to_string(),
                client_secret: "secret123".to_string(),
                access_token: Some("access_token123".to_string()),
                refresh_token: None,
            },
            rest_api: RestApiConfig {
                base_url: "https://api.tradier.com".to_string(),
                timeout: 30,
            },
            streaming: StreamingConfig {
                http_url: "https://stream.tradier.com/v1/markets/events".to_string(),
                websocket_url: "wss://ws.tradier.com/v1/markets/events".to_string(),
                reconnect_interval: 5,
            },
        };

        let display_output = config.to_string();
        let expected_json = json!({
            "credentials": {
                "client_id": "[REDACTED]",
                "client_secret": "[REDACTED]",
                "access_token": "[REDACTED]",
                "refresh_token": null
            },
            "rest_api": {
                "base_url": "https://api.tradier.com",
                "timeout": 30
            },
            "streaming": {
                "http_url": "https://stream.tradier.com/v1/markets/events",
                "websocket_url": "wss://ws.tradier.com/v1/markets/events",
                "reconnect_interval": 5
            }
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }
}