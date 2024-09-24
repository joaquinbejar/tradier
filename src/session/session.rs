use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio;
use tracing::{debug};
use reqwest::Client as HttpClient;
use crate::config::base::Config;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{DateTime, Duration, Utc};
use crate::constants::TRADIER_SESSION_TIMEOUT;

static SESSION_EXISTS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct Session {
    pub session_type: SessionType,
    pub stream_info: StreamInfo,
    created_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub stream: StreamInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub url: String,
    #[serde(rename = "sessionid")]
    pub session_id: String,
}


#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    Market,
    Account,
}

impl Session {
    pub async fn new(session_type: SessionType, config: &Config) -> Result<Self, Box<dyn Error>> {
        if SESSION_EXISTS.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err("Session already exists".into());
        }

        let client = HttpClient::new();
        let url = match session_type {
            SessionType::Market => format!("{}/v1/markets/events/session", config.rest_api.base_url),
            SessionType::Account => format!("{}/v1/accounts/events/session", config.rest_api.base_url),
        };

        let access_token = config.credentials.access_token.as_ref()
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
            SESSION_EXISTS.store(false, Ordering::SeqCst);  // Reset the flag if session creation fails
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


    pub fn is_expired(&self) -> bool {
        Utc::now() - self.created_at > Duration::minutes(TRADIER_SESSION_TIMEOUT)
    }

    pub fn get_websocket_url(&self) -> &str {
        &self.stream_info.url
    }

    pub fn get_session_id(&self) -> &str {
        &self.stream_info.session_id
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        SESSION_EXISTS.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests_session {
    use super::*;
    use mockito::Server;
    use crate::config::base::{Credentials, RestApiConfig, StreamingConfig};
    use std::sync::Once;
    use serial_test::serial;

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
        assert_eq!(session.get_websocket_url(), "wss://ws.tradier.com/v1/accounts/events");
        assert_eq!(session.get_session_id(), "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3");

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial]  // Se ejecuta de forma secuencial
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
        assert_eq!(session.get_websocket_url(), "https://stream.tradier.com/v1/markets/events");
        assert_eq!(session.get_session_id(), "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3");

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial]  // Se ejecuta de forma secuencial
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
        assert!(session2.is_err(), "Should not be able to create a second session");
        assert!(session2.unwrap_err().to_string().contains("Session already exists"));

        // Drop the first session to reset the state
        drop(session1);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[serial]  // Se ejecuta de forma secuencial
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
