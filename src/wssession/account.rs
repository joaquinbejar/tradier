//! # Tradier Account WebSocket Module
//!
//! This module provides functionality to interact with the Tradier Account WebSocket API, enabling real-time
//! streaming of account events such as order status updates, executions, and balance changes.
//!
//! ## Overview
//!
//! Tradier’s Account WebSocket API is designed to deliver immediate updates on account activities to authorized
//! clients. This API is useful for applications that need real-time insights into account status, including:
//! - Order placement, cancellation, and execution events
//! - Updates on account balances and positions
//!
//! An `AccountSession` in this module wraps Tradier’s WebSocket session specifically for account-related events,
//! ensuring efficient management of the WebSocket connection and enabling clients to receive streaming data
//! continuously. The `SessionManager` is used to enforce a singleton pattern, allowing only one session at a time
//! to prevent overlapping WebSocket connections.
//!
//! ## Usage
//!
//! To set up an account WebSocket session, initialize a `Config` with the necessary API credentials, and create
//! a `SessionManager` to manage the session state. Then, instantiate an `AccountSession` to begin streaming
//! account events from Tradier.
//!
//! ### Example
//!
//! ```no_run
//! use tradier::Config;
//! use tradier::wssession::AccountSession;
//! use tracing::info;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Load configuration with API credentials
//!     let config = Config::new();
//!     
//!     // Establish an account session
//!     match AccountSession::new(&config).await {
//!         Ok(account_session) => {
//!             info!("Account WebSocket session established with ID: {}", account_session.get_session_id());
//!             
//!             // Use `account_session` to start receiving account updates
//!             let websocket_url = account_session.get_websocket_url();
//!             println!("Connect to WebSocket at: {}", websocket_url);
//!         }
//!         Err(e) => {
//!             eprintln!("Failed to create account WebSocket session: {}", e);
//!         }
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! The `AccountSession` creation will return an error in cases such as:
//! - Missing or invalid API credentials
//! - Network connectivity issues preventing WebSocket connection establishment
//! - Attempting to create multiple sessions simultaneously, which is restricted by `SessionManager`
//!
//! For additional details on the API, refer to the [Tradier Account WebSocket documentation](https://documentation.tradier.com/brokerage-api/streaming/wss-account-websocket).

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tracing::{debug, info, trace, warn};
use tungstenite::Message;
use url::Url;

use crate::Config;
use crate::wssession::account_events::AccountEvent;
use crate::wssession::session::{Session, SessionType};
use crate::{Error, Result};

use self::session_manager::{GLOBAL_SESSION_MANAGER, SessionManager};

use super::session_manager;

/// Filter for account WebSocket events. See
/// <https://documentation.tradier.com/brokerage-api/streaming/wss-account-websocket>.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(try_from = "&str", into = "String")]
#[non_exhaustive]
pub enum AccountSessionEvent {
    /// Order status / lifecycle updates.
    Order,
    /// Position changes.
    Position,
    /// Trade events.
    Trade,
    /// Fill (execution) events.
    Fill,
    /// Account dropped / session ended.
    Drop,
}

impl AsRef<str> for AccountSessionEvent {
    #[inline]
    fn as_ref(&self) -> &'static str {
        match self {
            AccountSessionEvent::Order => "order",
            AccountSessionEvent::Position => "position",
            AccountSessionEvent::Trade => "trade",
            AccountSessionEvent::Fill => "fill",
            AccountSessionEvent::Drop => "drop",
        }
    }
}

impl TryFrom<&str> for AccountSessionEvent {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "order" => Ok(AccountSessionEvent::Order),
            "position" => Ok(AccountSessionEvent::Position),
            "trade" => Ok(AccountSessionEvent::Trade),
            "fill" => Ok(AccountSessionEvent::Fill),
            "drop" => Ok(AccountSessionEvent::Drop),
            _ => Err(Error::UnsupportedAccountEvent(value.to_owned())),
        }
    }
}

impl From<AccountSessionEvent> for String {
    fn from(val: AccountSessionEvent) -> Self {
        val.as_ref().to_owned()
    }
}

/// Payload for a Tradier Account WebSocket session.
///
/// The server expects `events` and `sessionid`. `excludeAccounts` is
/// optional and lets callers mask specific sub-accounts from the stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountSessionPayload<'a> {
    pub events: Cow<'a, [AccountSessionEvent]>,
    #[serde(rename = "sessionid")]
    pub session_id: Cow<'a, str>,
    #[serde(rename = "excludeAccounts", skip_serializing_if = "Option::is_none")]
    pub exclude_accounts: Option<Cow<'a, [String]>>,
}

#[bon::bon]
impl<'a> AccountSessionPayload<'a> {
    #[builder(builder_type(vis = "pub"))]
    fn new(
        events: &'a [AccountSessionEvent],
        session_id: &'a str,
        exclude_accounts: Option<&'a [String]>,
    ) -> Self {
        AccountSessionPayload {
            events: Cow::Borrowed(events),
            session_id: Cow::Borrowed(session_id),
            exclude_accounts: exclude_accounts.map(Cow::Borrowed),
        }
    }

    /// Converts the payload to a WebSocket `Message` for sending.
    ///
    /// # Errors
    /// Returns [`Error::JsonParsingError`] if the payload cannot be
    /// serialized as JSON.
    pub fn get_message(&self) -> Result<Message> {
        serde_json::to_string(self)
            .map(|s| Message::Text(s.into()))
            .map_err(Into::into)
    }
}

impl<'a> Display for AccountSessionPayload<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let events: Vec<&str> = self.events.iter().map(|e| e.as_ref()).collect();
        write!(
            f,
            "AccountSessionPayload {{ events: {:?}, session_id: {}, exclude_accounts: {:?} }}",
            events, self.session_id, self.exclude_accounts,
        )
    }
}

/// `AccountSession` is a wrapper around a `Session` specifically for account-level WebSocket interactions.
///
/// This struct is designed to interact with Tradier's Account WebSocket API, which provides real-time
/// updates on account activities such as order status changes and account balance modifications. It
/// encapsulates a `Session` initialized with `SessionType::Account`, offering convenient methods to
/// establish and manage WebSocket connections for account-related data.
///
/// For more details on the Tradier Account WebSocket API, see the official [documentation](https://documentation.tradier.com/brokerage-api/streaming/wss-account-websocket).
#[derive(Debug, Clone)]
pub struct AccountSession<'a>(Session<'a>);

impl<'a> AccountSession<'a> {
    /// Creates a new `AccountSession` using the provided configuration and session manager.
    ///
    /// This method establishes a WebSocket session that streams real-time account events from Tradier.
    /// Events may include updates to order statuses, executions, and account balances. Tradier requires
    /// an active session ID and authentication details in `Config` to access the WebSocket API.
    ///
    /// # Arguments
    ///
    /// * `session_manager` - A reference to `SessionManager`, ensuring only one session is active.
    /// * `config` - A reference to `Config`, which provides essential details such as API credentials.
    ///
    /// # Returns
    ///
    /// * `Ok(AccountSession)` - A new instance of `AccountSession` if the session creation is successful.
    /// * `Err(Box<dyn Error>)` - An error if session creation fails, e.g., due to network issues or invalid credentials.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tradier::Config;
    /// use tradier::wssession::AccountSession;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let account_session = AccountSession::new(&config)
    ///         .await
    ///         .expect("Failed to create account session");
    /// }
    /// ```
    pub async fn new(config: &Config) -> Result<Self> {
        Self::new_with_session_manager(config, &GLOBAL_SESSION_MANAGER).await
    }

    #[allow(dead_code)]
    async fn new_with_session_manager(
        config: &Config,
        session_manager: &'a SessionManager,
    ) -> Result<Self> {
        Ok(AccountSession(
            Session::new_with_session_manager(session_manager, SessionType::Account, config)
                .await?,
        ))
    }

    /// Retrieves the session ID associated with this account session.
    ///
    /// The session ID is a unique identifier provided by Tradier when a WebSocket connection is established.
    /// It is used to track and manage the session, especially useful for reconnecting or troubleshooting issues.
    ///
    /// # Returns
    ///
    /// * `&str` - The session ID as a string slice, uniquely identifying this session.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tradier::Config;
    /// use tradier::wssession::AccountSession;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let account_session = AccountSession::new(&config)
    ///         .await
    ///         .expect("Failed to create account session");
    ///     let session_id = account_session.get_session_id();
    ///     println!("Session ID: {}", session_id);
    /// }
    /// ```
    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    /// Retrieves the WebSocket URL for this account session.
    ///
    /// This URL is used to establish a WebSocket connection with Tradier's account-level streaming service.
    /// The WebSocket connection provides real-time streaming data, allowing applications to receive live
    /// updates on account events.
    ///
    /// # Returns
    ///
    /// * `&str` - The WebSocket URL as a string slice, which can be used to initiate the WebSocket connection.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tradier::Config;
    /// use tradier::wssession::AccountSession;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let account_session = AccountSession::new(&config)
    ///         .await
    ///         .expect("Failed to create account session");
    ///     let websocket_url = account_session.get_websocket_url();
    ///     println!("WebSocket URL: {}", websocket_url);
    /// }
    /// ```
    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }

    /// Opens the account WebSocket and returns an async [`Stream`] of
    /// typed [`AccountEvent`] values.
    ///
    /// Each upstream text frame is split on `\n` so a single frame that
    /// bundles multiple newline-delimited JSON events still decodes as
    /// separate items.
    ///
    /// Decode failures surface as `Err(Error::StreamDecodeError(...))`
    /// items and do NOT abort the stream. Peer-initiated `Close` ends
    /// the stream without yielding an error. `Ping` frames are answered
    /// automatically by `tokio_tungstenite`.
    ///
    /// Reconnect policy is the caller's responsibility.
    ///
    /// # Errors
    /// - [`Error::UrlParsingError`] if the session URL is invalid.
    /// - [`Error::WebSocketError`] if the handshake or initial payload
    ///   send fails.
    /// - [`Error::JsonParsingError`] if the subscription payload cannot
    ///   be serialized.
    ///
    /// After the stream is established, per-frame errors are yielded as
    /// `Err(_)` items of the stream rather than aborting early.
    pub async fn event_stream(
        &self,
        payload: AccountSessionPayload<'a>,
    ) -> Result<impl Stream<Item = Result<AccountEvent>>> {
        let uri = self.0.get_websocket_url();
        let url = Url::parse(uri)?;

        info!(url = %uri, "connecting to account stream");
        let (ws_stream, _) = connect_async(url.as_str()).await.map_err(Box::new)?;
        let (mut write, read) = ws_stream.split();

        let message = payload.get_message()?;
        write.send(message).await.map_err(Box::new)?;
        debug!("sent account subscription payload");

        Ok(super::ws_decode::ws_event_stream(
            read,
            AccountEvent::from_json,
        ))
    }

    /// Opens the account WebSocket, sends the payload, and streams events
    /// until the peer closes the connection or an error occurs.
    ///
    /// Kept for back-compat — internally drives [`Self::event_stream`]
    /// and logs each parsed event at `TRACE`, each decode error at
    /// `WARN`. Prefer [`Self::event_stream`] for new code.
    ///
    /// # Errors
    /// Same as [`Self::event_stream`].
    pub async fn ws_stream(&self, payload: AccountSessionPayload<'a>) -> Result<()> {
        let stream = self.event_stream(payload).await?;
        futures_util::pin_mut!(stream);
        while let Some(item) = stream.next().await {
            match item {
                Ok(event) => {
                    // Do not log the account number in clear — record only
                    // whether it was present on the event.
                    trace!(
                        has_account = event.account_number().is_some(),
                        "account event"
                    );
                }
                Err(Error::StreamDecodeError(_, err)) => {
                    warn!(error = %err, "decode failure on account frame, continuing");
                }
                Err(e) => {
                    warn!(error = %e, "account stream error, terminating");
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockito::Server;

    use super::*;
    use crate::{
        utils::tests::{
            ScriptedWsAction, create_test_config, free_tcp_port, scripted_websocket_server,
        },
        wssession::session_manager::SessionManager,
    };

    #[test]
    fn test_account_session_event_as_ref() {
        assert_eq!(AccountSessionEvent::Order.as_ref(), "order");
        assert_eq!(AccountSessionEvent::Position.as_ref(), "position");
        assert_eq!(AccountSessionEvent::Trade.as_ref(), "trade");
        assert_eq!(AccountSessionEvent::Fill.as_ref(), "fill");
        assert_eq!(AccountSessionEvent::Drop.as_ref(), "drop");
    }

    #[test]
    fn test_account_session_event_try_from_rejects_unknown() {
        let result = AccountSessionEvent::try_from("bogus");
        assert!(matches!(result, Err(Error::UnsupportedAccountEvent(_))));
    }

    #[test]
    fn test_account_session_payload_get_message_serializes_events_and_session_id() {
        let events = [AccountSessionEvent::Order, AccountSessionEvent::Fill];
        let payload = AccountSessionPayload::builder()
            .events(&events)
            .session_id("sess-123")
            .build();
        let msg = payload.get_message().expect("serialize");
        let tungstenite::Message::Text(text) = msg else {
            panic!("expected text frame");
        };
        assert!(text.contains("\"events\":[\"order\",\"fill\"]"));
        assert!(text.contains("\"sessionid\":\"sess-123\""));
        assert!(!text.contains("excludeAccounts"));
    }

    #[test]
    fn test_account_session_payload_get_message_includes_exclude_accounts_when_set() {
        let events = [AccountSessionEvent::Order];
        let excluded = ["VA1234".to_owned()];
        let payload = AccountSessionPayload::builder()
            .events(&events)
            .session_id("sess-123")
            .exclude_accounts(&excluded)
            .build();
        let msg = payload.get_message().expect("serialize");
        let tungstenite::Message::Text(text) = msg else {
            panic!("expected text frame");
        };
        assert!(text.contains("\"excludeAccounts\":[\"VA1234\"]"));
    }

    // This test breaks if you have valid Config env vars set. The method we have to fix this
    // is the `crate::utils::with_env_vars()` method, but this method doesn't support async
    // code, at the moment.
    #[tokio::test]
    async fn test_invalid_config() {
        let session_manager = SessionManager::default();
        let config = Config::new();

        let account_session =
            AccountSession::new_with_session_manager(&config, &session_manager).await;

        assert!(account_session.is_err())
    }

    #[tokio::test]
    async fn test_account_session_creation() {
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

        let config = create_test_config().server_url(&server.url()).finish();
        let session_manager = SessionManager::default();
        {
            let session = AccountSession::new_with_session_manager(&config, &session_manager)
                .await
                .unwrap();

            assert_eq!(
                session.get_websocket_url(),
                "wss://ws.tradier.com/v1/accounts/events"
            );
            assert_eq!(
                session.get_session_id(),
                "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
            );
        }
        mock.assert_async().await;
    }

    // ===== event_stream tests =====

    const ORDER_FRAME: &str = r#"{"event":"order","id":123,"status":"filled","account_number":"VA1234","symbol":"SPY","quantity":100.0,"executed_quantity":100.0}"#;
    const FILL_FRAME: &str = r#"{"event":"fill","order_id":123,"account_number":"VA1234","symbol":"SPY","side":"buy","quantity":100.0,"price":281.12}"#;
    const POSITION_FRAME: &str = r#"{"event":"position","account_number":"VA1234","symbol":"SPY","quantity":100.0,"cost_basis":28112.0}"#;
    const BALANCE_FRAME: &str = r#"{"event":"balance","account_number":"VA1234","total_equity":100000.0,"total_cash":50000.0}"#;
    const TRADE_FRAME: &str = r#"{"event":"trade","account_number":"VA1234","symbol":"SPY","side":"buy","quantity":100.0,"price":281.12}"#;
    const DROP_FRAME: &str =
        r#"{"event":"drop","account_number":"VA1234","reason":"session expired"}"#;
    const MALFORMED_FRAME: &str = r#"{"event":"order","id":123,"status":"#;

    /// Boots an account session wired to a leaked `SessionManager` and
    /// `Config` so the returned session satisfies `'static`.
    async fn build_account_session_against_port(port: u16) -> AccountSession<'static> {
        let server = Box::leak(Box::new(Server::new_async().await));
        let json_data = format!(
            r#"
        {{
            "stream": {{
                "url": "ws://127.0.0.1:{port}/v1/accounts/events",
                "sessionid": "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
            }}
        }}
        "#
        );
        server
            .mock("POST", "/v1/accounts/events/session")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_data)
            .create_async()
            .await;

        let config = Box::leak(Box::new(
            create_test_config().server_url(&server.url()).finish(),
        ));
        let session_manager: &'static SessionManager =
            Box::leak(Box::new(SessionManager::default()));
        AccountSession::new_with_session_manager(config, session_manager)
            .await
            .expect("account session")
    }

    #[tokio::test]
    async fn test_account_event_stream_yields_typed_events_in_order() {
        let port = free_tcp_port();
        let session = build_account_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(ORDER_FRAME),
                ScriptedWsAction::SendText(FILL_FRAME),
                ScriptedWsAction::SendText(POSITION_FRAME),
                ScriptedWsAction::SendText(BALANCE_FRAME),
                ScriptedWsAction::SendText(TRADE_FRAME),
                ScriptedWsAction::SendText(DROP_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let events = [AccountSessionEvent::Order];
        let payload = AccountSessionPayload::builder()
            .events(&events)
            .session_id(session.get_session_id())
            .build();

        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<AccountEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 6, "expected 6 events, got {collected:?}");
        assert!(matches!(collected[0], Ok(AccountEvent::Order(_))));
        assert!(matches!(collected[1], Ok(AccountEvent::Fill(_))));
        assert!(matches!(collected[2], Ok(AccountEvent::Position(_))));
        assert!(matches!(collected[3], Ok(AccountEvent::Balance(_))));
        assert!(matches!(collected[4], Ok(AccountEvent::Trade(_))));
        assert!(matches!(collected[5], Ok(AccountEvent::Drop(_))));
    }

    #[tokio::test]
    async fn test_account_event_stream_malformed_frame_yields_decode_error_and_continues() {
        let port = free_tcp_port();
        let session = build_account_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(ORDER_FRAME),
                ScriptedWsAction::SendText(MALFORMED_FRAME),
                ScriptedWsAction::SendText(FILL_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let events = [AccountSessionEvent::Order];
        let payload = AccountSessionPayload::builder()
            .events(&events)
            .session_id(session.get_session_id())
            .build();
        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<AccountEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 3, "expected 3 items, got {collected:?}");
        assert!(matches!(collected[0], Ok(AccountEvent::Order(_))));
        assert!(matches!(collected[1], Err(Error::StreamDecodeError(_, _))));
        assert!(matches!(collected[2], Ok(AccountEvent::Fill(_))));
    }

    #[tokio::test]
    async fn test_account_event_stream_peer_close_terminates_without_error() {
        let port = free_tcp_port();
        let session = build_account_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(ORDER_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let events = [AccountSessionEvent::Order];
        let payload = AccountSessionPayload::builder()
            .events(&events)
            .session_id(session.get_session_id())
            .build();
        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<AccountEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 1);
        assert!(collected[0].is_ok());
    }
}
