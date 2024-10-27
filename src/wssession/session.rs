use crate::config::Config;
use crate::constants::TRADIER_SESSION_TIMEOUT;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

/// A global flag used to enforce a singleton session. 
/// This flag ensures that only one session instance can exist at any given time.
static SESSION_EXISTS: AtomicBool = AtomicBool::new(false);

/// Represents a Tradier API session, handling WebSocket streaming configuration for either 
/// account or market data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Session {
    /// The type of session, either `Account` or `Market`.
    pub session_type: SessionType,
    /// Contains information about the WebSocket stream, including URL and session ID.
    pub stream_info: StreamInfo,
    created_at: DateTime<Utc>,
}

/// Response structure for the Tradier API session request. Holds the stream information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    /// Contains the WebSocket stream information such as URL and session ID.
    pub stream: StreamInfo,
}

/// Holds information about the WebSocket stream, including the URL and session ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    /// WebSocket URL for the stream.
    pub url: String,
    /// Unique identifier for the session, used to maintain connection context.
    #[serde(rename = "sessionid")]
    pub session_id: String,
}


/// Specifies the type of Tradier API session, either `Market` for market data
/// or `Account` for account-related data.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    Market,
    Account,
}

impl Session {
    /// Creates a new `Session` instance based on the specified session type and configuration.
    ///
    /// - **session_type**: Specifies the type of session, either `SessionType::Market` or `SessionType::Account`.
    /// - **config**: Reference to `Config`, providing required details like base URLs and access tokens.
    ///
    /// # Returns
    /// - `Ok(Session)`: A new `Session` if successful.
    /// - `Err(Box<dyn Error>)`: If session creation fails due to an existing session, missing token, or API error.
    ///
    /// # Errors
    /// - Fails if a session already exists (singleton restriction).
    /// - Fails if the access token is missing or invalid.
    /// - Fails if the API request encounters network issues or the API returns an error status.
    pub async fn new(session_type: SessionType, config: &Config) -> Result<Self, Box<dyn Error>> {
        if SESSION_EXISTS
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err("Session already exists".into());
        }

        let client = HttpClient::new();
        let url = match session_type {
            SessionType::Market => {
                format!("{}/v1/markets/events/session", config.rest_api.base_url)
            }
            SessionType::Account => {
                format!("{}/v1/accounts/events/session", config.rest_api.base_url)
            }
        };
        debug!("Url to use to get the Session ID: {}", url);

        let access_token = config
            .credentials
            .access_token
            .as_ref()
            .ok_or("Access token not found in configuration")?;

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        debug!("Response status: {}", status);
        debug!("Response headers: {:?}", headers);

        let body = response.text().await?;
        debug!("Response body: {}", body);

        if status.is_success() {
            let session_response: SessionResponse = serde_json::from_str(&body)?;
            Ok(Session {
                session_type,
                stream_info: session_response.stream,
                created_at: Utc::now(),
            })
        } else {
            SESSION_EXISTS.store(false, Ordering::SeqCst); // Reset the flag if session creation fails
            Err(format!(
                "Failed to create {} session. Status: {}. Body: {}",
                match session_type {
                    SessionType::Market => "market",
                    SessionType::Account => "account",
                },
                status,
                body
            )
            .into())
        }
    }

    /// Checks if the session has expired based on the configured session timeout.
    ///
    /// # Returns
    /// - `true` if the session duration exceeds `TRADIER_SESSION_TIMEOUT`, otherwise `false`.
    #[allow(dead_code)]
    pub fn is_expired(&self) -> bool {
        Utc::now() - self.created_at > Duration::minutes(TRADIER_SESSION_TIMEOUT)
    }

    /// Retrieves the WebSocket URL associated with the session.
    ///
    /// # Returns
    /// - `&str`: The WebSocket URL.
    pub fn get_websocket_url(&self) -> &str {
        &self.stream_info.url
    }

    /// Retrieves the session ID associated with the session.
    ///
    /// # Returns
    /// - `&str`: The session ID.
    pub fn get_session_id(&self) -> &str {
        &self.stream_info.session_id
    }
}

impl Drop for Session {
    /// Custom drop implementation for `Session` that resets the `SESSION_EXISTS` flag, allowing
    /// a new session to be created once the current one is dropped.
    fn drop(&mut self) {
        SESSION_EXISTS.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests_session {
    use super::*;
    use crate::config::{Credentials, RestApiConfig, StreamingConfig};
    use mockito::Server;
    use serial_test::serial;
    use std::sync::Once;

    static INIT: Once = Once::new();

    // Helper function to create a test config
    fn create_test_config(server_url: &str, is_sandbox: bool) -> Config {
        Config {
            credentials: Credentials {
                client_id: "test_id".to_string(),
                client_secret: "test_secret".to_string(),
                access_token: Some("test_token".to_string()),
                refresh_token: None,
            },
            rest_api: RestApiConfig {
                base_url: if is_sandbox {
                    "https://sandbox.tradier.com".to_string()
                } else {
                    server_url.to_string()
                },
                timeout: 30,
            },
            streaming: StreamingConfig {
                http_base_url: "".to_string(),
                ws_base_url: "".to_string(),
                events_path: "".to_string(),
                reconnect_interval: 5,
            },
        }
    }

    fn setup() {
        INIT.call_once(|| {
            // Reset the SESSION_EXISTS flag before each test
            SESSION_EXISTS.store(false, Ordering::SeqCst);
        });
    }

    #[tokio::test]
    #[serial]
    async fn test_account_session_creation() {
        setup();
        let mut server = Server::new_async().await;
        let json_data = r#"
        {
            "stream": {
                "url": "wss://ws.tradier.com/v1/accounts/events",
                "sessionid": "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
            }
        }
        "#;
        let mock = server
            .mock("POST", "/v1/accounts/events/session")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_data)
            .create_async()
            .await;

        let config = create_test_config(&server.url(), false);
        let session = Session::new(SessionType::Account, &config).await.unwrap();

        assert_eq!(session.session_type, SessionType::Account);
        assert_eq!(
            session.get_websocket_url(),
            "wss://ws.tradier.com/v1/accounts/events"
        );
        assert_eq!(
            session.get_session_id(),
            "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial] // Se ejecuta de forma secuencial
    async fn test_market_session_creation() {
        setup();
        let mut server = Server::new_async().await;
        let json_data = r#"
        {
            "stream": {
                "url": "https://stream.tradier.com/v1/markets/events",
                "sessionid": "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
            }
        }
        "#;
        let mock = server
            .mock("POST", "/v1/markets/events/session")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_data)
            .create_async()
            .await;

        let config = create_test_config(&server.url(), false);
        let session = Session::new(SessionType::Market, &config).await.unwrap();

        assert_eq!(session.session_type, SessionType::Market);
        assert_eq!(
            session.get_websocket_url(),
            "https://stream.tradier.com/v1/markets/events"
        );
        assert_eq!(
            session.get_session_id(),
            "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial] // Se ejecuta de forma secuencial
    async fn test_multiple_session_creation() {
        setup();
        let mut server = Server::new_async().await;
        let json_data = r#"
        {
            "stream": {
                "url": "https://stream.tradier.com/v1/markets/events",
                "sessionid": "test_session_id"
            }
        }
        "#;
        let mock = server
            .mock("POST", "/v1/markets/events/session")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_data)
            .expect(1)
            .create_async()
            .await;

        let config = create_test_config(&server.url(), false);

        // Create first session
        let session1 = Session::new(SessionType::Market, &config).await.unwrap();

        // Attempt to create second session immediately (should fail)
        let session2 = Session::new(SessionType::Market, &config).await;
        assert!(
            session2.is_err(),
            "Should not be able to create a second session"
        );
        assert!(session2
            .unwrap_err()
            .to_string()
            .contains("Session already exists"));

        // Drop the first session to reset the state
        drop(session1);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial] // Se ejecuta de forma secuencial
    async fn test_content_length_header() {
        setup();
        let mut server = Server::new_async().await;
        let json_data = r#"
        {
            "stream": {
                "url": "https://stream.tradier.com/v1/markets/events",
                "sessionid": "test_session_id"
            }
        }
        "#;
        let mock = server
            .mock("POST", "/v1/markets/events/session")
            .match_header("Content-Length", "0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_data)
            .create_async()
            .await;

        let config = create_test_config(&server.url(), false);
        let session = Session::new(SessionType::Market, &config).await.unwrap();

        assert_eq!(session.session_type, SessionType::Market);
        mock.assert_async().await;
    }
}
