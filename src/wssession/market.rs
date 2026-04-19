use crate::config::Config;
use crate::wssession::events::MarketEvent;
use crate::wssession::session::{Session, SessionType};
use crate::{Error, Result};
use futures_util::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use tokio_tungstenite::connect_async;
use tracing::{debug, info, trace, warn};
use tungstenite::Message;
use url::Url;

use super::session_manager::{SessionManager, GLOBAL_SESSION_MANAGER};

/// `MarketSessionFilter` represents the possible filters for a market WebSocket session.
///
/// Options include:
/// - `TRADE`: Filters trade-related events.
/// - `QUOTE`: Filters quote-related events.
/// - `SUMMARY`: Filters summary events.
/// - `TIMESALE`: Filters time sale events.
/// - `TRADEX`: Filters extended trade events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(into = "String", try_from = "&str")]
pub enum MarketSessionFilter {
    TRADE,
    QUOTE,
    SUMMARY,
    TIMESALE,
    TRADEX,
}

impl AsRef<str> for MarketSessionFilter {
    /// Returns the string representation of each filter option.
    ///
    /// # Returns
    /// - `&'static str`: The string corresponding to the filter type.
    fn as_ref(&self) -> &'static str {
        match self {
            MarketSessionFilter::TRADE => "trade",
            MarketSessionFilter::QUOTE => "quote",
            MarketSessionFilter::SUMMARY => "summary",
            MarketSessionFilter::TIMESALE => "timesale",
            MarketSessionFilter::TRADEX => "tradex",
        }
    }
}
impl TryFrom<&str> for MarketSessionFilter {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "trade" => Ok(MarketSessionFilter::TRADE),
            "quote" => Ok(MarketSessionFilter::QUOTE),
            "summary" => Ok(MarketSessionFilter::SUMMARY),
            "timesale" => Ok(MarketSessionFilter::TIMESALE),
            "tradex" => Ok(MarketSessionFilter::TRADEX),
            _ => Err(Error::UnsupportedMarketFilter(value.to_owned())),
        }
    }
}

impl From<MarketSessionFilter> for String {
    fn from(val: MarketSessionFilter) -> Self {
        val.as_ref().to_owned()
    }
}

/// Payload for a Tradier Market WebSocket session. This structure defines the settings
/// for a market session, including symbols, filters, and various optional parameters.
///
/// Fields:
/// - `symbols`: Vector of symbol strings to subscribe to in the market session.
/// - `filter`: Optional vector of `MarketSessionFilter` values to apply to the session.
/// - `session_id`: Unique session identifier.
/// - `linebreak`: Optional boolean to insert a linebreak after a completed payload.
/// - `valid_only`: Optional boolean that, if set to `true`, inludes only ticks considered valid by exchanges.
/// - `advanced_details`: Optional boolean for additional data in advanced detail format (applicable to timesale payloads only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSessionPayload<'a> {
    pub symbols: Cow<'a, [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Cow<'a, [MarketSessionFilter]>>,
    #[serde(rename = "sessionid")]
    pub session_id: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linebreak: Option<bool>,
    #[serde(rename = "validOnly", skip_serializing_if = "Option::is_none")]
    pub valid_only: Option<bool>,
    #[serde(rename = "advancedDetails", skip_serializing_if = "Option::is_none")]
    pub advanced_details: Option<bool>,
}

#[bon::bon]
impl<'a> MarketSessionPayload<'a> {
    /// Constructs a new `MarketSessionPayload` with default filter settings.
    ///
    /// # Arguments
    /// - `symbols`: A vector of symbols to subscribe to.
    /// - `session_id`: A unique session identifier for the WebSocket connection.
    ///
    /// # Returns
    /// - `Self`: A new `MarketSessionPayload` instance with default filter and optional settings.
    #[builder(builder_type(vis = "pub"))]
    fn new(
        symbols: &'a [String],
        filters: Option<&'a [MarketSessionFilter]>,
        session_id: &'a str,
        linebreak: Option<bool>,
        valid_only: Option<bool>,
        advanced_details: Option<bool>,
    ) -> Self {
        MarketSessionPayload {
            symbols: Cow::Borrowed(symbols),
            filters: filters.map(Cow::Borrowed),
            session_id: Cow::Borrowed(session_id),
            linebreak,
            valid_only,
            advanced_details,
        }
    }

    pub fn recommended(symbols: &'a [String], session_id: &'a str) -> Self {
        Self::builder()
            .symbols(symbols)
            .filters(&[MarketSessionFilter::QUOTE])
            .session_id(session_id)
            .linebreak(true)
            .valid_only(false)
            .advanced_details(false)
            .build()
    }

    /// Converts the payload to a WebSocket `Message` for sending.
    ///
    /// # Returns
    /// - `Ok(Message)`: The WebSocket message if serialization is successful.
    /// - `Err(Box<dyn Error>)`: An error if serialization fails.
    pub fn get_message(&self) -> Result<Message> {
        serde_json::to_string(self)
            .map(|s| Message::Text(s.into()))
            .map_err(Into::into)
    }
}

impl<'a> Display for MarketSessionPayload<'a> {
    /// Implements `Display` for `MarketSessionPayload` to show formatted details.
    ///
    /// Format includes:
    /// - Symbols
    /// - Filters
    /// - Session ID
    /// - Optional settings (linebreak, valid_only, advanced_details)
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let filter_str: Vec<String> = self.filters.as_ref().map_or(vec![], |filters| {
            filters.iter().map(|f| f.as_ref().to_string()).collect()
        });

        write!(
            f,
            "MarketSessionPayload {{ symbols: {:?}, filter: {:?}, session_id: {}, linebreak: {}, valid_only: {}, advanced_details: {} }}",
            self.symbols,
            filter_str,
            self.session_id,
            self.linebreak.map_or("None".to_string(), |v| v.to_string()),
            self.valid_only.map_or("None".to_string(), |v| v.to_string()),
            self.advanced_details.map_or("None".to_string(), |v| v.to_string())
        )
    }
}

/// Represents a market session that can be used to receive streaming data
/// from the Tradier WebSocket API.
pub struct MarketSession<'a>(Session<'a>);

impl<'a> MarketSession<'a> {
    /// Creates a new `MarketSession` using the specified configuration.
    ///
    /// # Arguments
    /// - `config`: A reference to the `Config` structure containing API settings.
    ///
    /// # Returns
    /// - `Ok(MarketSession)`: A `MarketSession` instance if the session was created successfully.
    /// - `Err(Box<dyn Error>)`: If session creation fails.
    pub async fn new(config: &Config) -> Result<Self> {
        Self::new_with_session_manager(config, &GLOBAL_SESSION_MANAGER).await
    }

    async fn new_with_session_manager(
        config: &Config,
        session_manager: &'a SessionManager,
    ) -> Result<Self> {
        Ok(MarketSession(
            Session::new_with_session_manager(session_manager, SessionType::Market, config).await?,
        ))
    }

    /// Retrieves the session ID associated with the `MarketSession`.
    ///
    /// # Returns
    /// - `&str`: The session ID string.
    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    /// Retrieves the WebSocket URL for the `MarketSession`.
    ///
    /// # Returns
    /// - `&str`: The WebSocket URL string.
    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }

    /// Opens the market WebSocket and returns an async [`Stream`] of typed
    /// [`MarketEvent`] values.
    ///
    /// Each upstream text frame is split on `\n` so a single frame that
    /// happens to bundle multiple newline-delimited JSON events is still
    /// handled correctly (Tradier's current contract delivers one event
    /// per frame, but this keeps the decoder future-proof).
    ///
    /// Decode failures surface as `Err(Error::StreamDecodeError(...))`
    /// items and do NOT abort the stream — the decoder continues reading
    /// subsequent frames. Peer-initiated `Close` frames terminate the
    /// stream without yielding an error. `Ping` frames are answered
    /// automatically by `tokio_tungstenite`; if the automatic pong send
    /// fails, the failure is surfaced as `Err(Error::WebSocketError(...))`
    /// and the stream terminates.
    ///
    /// Reconnect policy is the caller's responsibility — see the example
    /// under `examples/auth_websocket_example.rs`.
    ///
    /// # Errors
    /// - [`Error::UrlParsingError`] if the session URL is malformed.
    /// - [`Error::WebSocketError`] if the handshake or initial payload
    ///   send fails.
    /// - [`Error::JsonParsingError`] if the subscription payload cannot
    ///   be serialized.
    ///
    /// After the stream is established, per-frame errors are yielded as
    /// `Err(_)` items of the stream rather than aborting early.
    pub async fn event_stream(
        &self,
        payload: MarketSessionPayload<'a>,
    ) -> Result<impl Stream<Item = Result<MarketEvent>>> {
        let uri = self.0.get_websocket_url();
        let url = Url::parse(uri)?;

        info!(url = %uri, "connecting to market stream");
        let (ws_stream, _) = connect_async(url.as_str()).await.map_err(Box::new)?;
        let (mut write, read) = ws_stream.split();

        let message = payload.get_message()?;
        write.send(message).await.map_err(Box::new)?;
        debug!("sent market subscription payload");

        Ok(market_event_stream(read))
    }

    /// Initiates a WebSocket connection and streams data based on the provided payload.
    ///
    /// Kept for back-compat — internally drives [`Self::event_stream`] and
    /// logs each parsed event at `TRACE`, each decode error at `WARN`.
    /// Prefer [`Self::event_stream`] for new code.
    ///
    /// # Errors
    /// Same as [`Self::event_stream`].
    pub async fn ws_stream(&self, payload: MarketSessionPayload<'a>) -> Result<()> {
        let stream = self.event_stream(payload).await?;
        futures_util::pin_mut!(stream);
        while let Some(item) = stream.next().await {
            match item {
                Ok(event) => {
                    trace!(symbol = event.symbol(), "market event");
                }
                Err(Error::StreamDecodeError(_, err)) => {
                    warn!(error = %err, "decode failure on market frame, continuing");
                }
                Err(e) => {
                    warn!(error = %e, "market stream error, terminating");
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

/// Internal newline-delimited [`MarketEvent`] decoder over a split
/// WebSocket read half.
///
/// Text frames are split on `\n`, each non-empty line is decoded into a
/// [`MarketEvent`]. Decode errors are yielded as `Err(_)` items but do
/// not terminate the stream. `Close` frames end the stream cleanly.
#[inline]
fn market_event_stream<S>(read: S) -> impl Stream<Item = Result<MarketEvent>>
where
    S: Stream<Item = std::result::Result<Message, tungstenite::Error>> + Unpin,
{
    super::ws_decode::ws_event_stream(read, MarketEvent::from_json)
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::tests::{
            create_test_config, free_tcp_port, mock_websocket_server, scripted_websocket_server,
            ScriptedWsAction,
        },
        wssession::session_manager::SessionManager,
    };

    use super::*;
    use mockito::Server;

    #[test]
    fn test_market_session_filter_as_ref() {
        assert_eq!(MarketSessionFilter::TRADE.as_ref(), "trade");
        assert_eq!(MarketSessionFilter::QUOTE.as_ref(), "quote");
        assert_eq!(MarketSessionFilter::SUMMARY.as_ref(), "summary");
        assert_eq!(MarketSessionFilter::TIMESALE.as_ref(), "timesale");
        assert_eq!(MarketSessionFilter::TRADEX.as_ref(), "tradex");
    }

    #[test]
    fn test_market_session_filter_try_from_str() {
        assert_eq!(
            MarketSessionFilter::try_from("trade").unwrap(),
            MarketSessionFilter::TRADE
        );
        assert_eq!(
            MarketSessionFilter::try_from("quote").unwrap(),
            MarketSessionFilter::QUOTE
        );

        let invalid_result = MarketSessionFilter::try_from("invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_market_session_filter_from() {
        let filter = MarketSessionFilter::TRADE;
        let filter_str: String = filter.into();
        assert_eq!(filter_str, "trade");
    }

    #[test]
    fn test_market_session_filter_serde_serialization() {
        let filter = MarketSessionFilter::TRADE;
        let json = serde_json::to_string(&filter).unwrap();
        assert_eq!(json, "\"trade\"");

        let deserialized: MarketSessionFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MarketSessionFilter::TRADE);
    }

    #[test]
    fn test_market_session_payload_recommended() {
        let symbols = vec!["AAPL".to_string(), "TSLA".to_string()];
        let session_id = "test-session-id".to_string();

        let payload = MarketSessionPayload::recommended(&symbols, &session_id);

        assert_eq!(payload.symbols, symbols);
        assert_eq!(
            payload.filters.as_deref(),
            Some(&[MarketSessionFilter::QUOTE][..])
        );
        assert_eq!(payload.session_id, session_id);
        assert_eq!(payload.linebreak, Some(true));
        assert_eq!(payload.valid_only, Some(false));
        assert_eq!(payload.advanced_details, Some(false));
    }

    #[test]
    fn test_market_session_payload_get_message_full_payload() {
        let symbols = ["AAPL".to_string(), "GOOGL".to_string()];
        let filters = vec![MarketSessionFilter::TRADE, MarketSessionFilter::SUMMARY];
        let session_id = "session-12345";

        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .filters(&filters)
            .session_id(session_id)
            .linebreak(true)
            .valid_only(false)
            .advanced_details(true)
            .build();

        let message_result = payload.get_message();
        assert!(
            message_result.is_ok(),
            "Expected message creation to succeed"
        );

        let message = message_result.unwrap();

        if let tungstenite::Message::Text(serialized) = message {
            assert!(serialized.contains("\"symbols\":[\"AAPL\",\"GOOGL\"]"));
            assert!(serialized.contains("\"filters\":[\"trade\",\"summary\"]"));
            assert!(serialized.contains("\"sessionid\":\"session-12345\""));
            assert!(serialized.contains("\"linebreak\":true"));
            assert!(serialized.contains("\"validOnly\":false"));
            assert!(serialized.contains("\"advancedDetails\":true"));
        } else {
            panic!("Expected a text WebSocket message, got {:?}", message);
        }
    }

    #[test]
    fn test_market_session_payload_display() {
        let symbols = ["AAPL".to_string(), "MSFT".to_string()];
        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .filters(&[MarketSessionFilter::TRADE, MarketSessionFilter::QUOTE])
            .session_id("display-session")
            .advanced_details(true)
            .build();

        let display_output = format!("{}", payload);
        assert!(display_output.contains("symbols: [\"AAPL\", \"MSFT\"]"));
        assert!(display_output.contains("filter: [\"trade\", \"quote\"]"));
        assert!(display_output.contains("session_id: display-session"));
        assert!(display_output.contains("linebreak: None"));
        assert!(display_output.contains("valid_only: None"));
        assert!(display_output.contains("advanced_details: true"));
    }

    #[tokio::test]
    async fn test_market_session_creation() {
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

        let config = create_test_config().server_url(&server.url()).finish();
        let session_manager = SessionManager::default();
        let session = MarketSession::new_with_session_manager(&config, &session_manager)
            .await
            .unwrap();

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
    async fn test_stream_some_data() {
        let expected_session_id = "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3";
        let expected_ws_host = "127.0.0.1";
        let expected_ws_port = 9999u16;
        let expected_ws_event_path = "/v1/markets/events";
        let expected_ws_url = format!(
            "ws://{}:{}/{}",
            expected_ws_host, expected_ws_port, expected_ws_event_path
        );
        let expected_symbols = ["AAPL".to_owned(), "C".to_owned()];
        let expected_ws_request = MarketSessionPayload::builder()
            .session_id(expected_session_id)
            .symbols(&expected_symbols)
            .build();
        let mut server = Server::new_async().await;
        let json_data = format!(
            r#"
        {{
            "stream": {{
                "url": "{}",
                "sessionid": "{}"
            }}
        }}
        "#,
            expected_ws_url, expected_session_id
        );

        let expected_event = r#"
        {{
            "type": "quote",
            "symbol": "C",
            "bid": 281.84,
            "bidsz": 60,
            "bidexch": "M",
            "biddate": "1557757189000",
            "ask": 281.85,
            "asksz": 6,
            "askexch": "Z",
            "askdate": "1557757190000"
        }}"#;
        let _mock = server
            .mock("POST", "/v1/markets/events/session")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_data)
            .create_async()
            .await;

        let config = create_test_config()
            .server_url(&server.url())
            .web_socket_url(&format!("ws://{}:{}", expected_ws_host, expected_ws_port))
            .web_socket_path(expected_ws_event_path)
            .finish();
        let session_manager = SessionManager::default();
        mock_websocket_server()
            .address(expected_ws_host, expected_ws_port)
            .expected_request(expected_ws_request)
            .expected_response(expected_event)
            .create()
            .await;

        let market_session =
            MarketSession::new_with_session_manager(&config, &session_manager).await;
        assert!(market_session.is_ok());
        let market_session = market_session.unwrap();
        let result = market_session
            .ws_stream(
                MarketSessionPayload::builder()
                    .session_id(market_session.get_session_id())
                    .symbols(&expected_symbols)
                    .build(),
            )
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_market_session_filter_invalid_input() {
        let invalid_filters = vec!["invalid", "unknown", "random"];

        for invalid_filter in invalid_filters {
            let result = MarketSessionFilter::try_from(invalid_filter);
            assert!(
                result.is_err(),
                "Expected error for invalid filter '{}', but got {:?}",
                invalid_filter,
                result
            );

            if let Err(Error::UnsupportedMarketFilter(filter)) = result {
                assert_eq!(filter, invalid_filter.to_owned());
            } else {
                panic!(
                    "Expected UnsupportedMarketFilter error, but got {:?}",
                    result
                );
            }
        }
    }

    // ===== event_stream tests =====
    //
    // These tests stand up a scripted local WebSocket server (no TLS) and
    // exercise the real framing through `MarketSession::event_stream`.

    const QUOTE_FRAME: &str = r#"{"type":"quote","symbol":"C","bid":281.84,"bidsz":60,"bidexch":"M","biddate":"1557757189000","ask":281.85,"asksz":6,"askexch":"Z","askdate":"1557757190000"}"#;
    const TRADE_FRAME: &str = r#"{"type":"trade","symbol":"SPY","exch":"Q","price":"281.1200","size":"100","cvol":"34507070","date":"1557757204760","last":"281.1200"}"#;
    const SUMMARY_FRAME: &str = r#"{"type":"summary","symbol":"SPY","open":"284.01","high":"284.42","low":"280.51","prevClose":"287.59"}"#;
    const TIMESALE_FRAME: &str = r#"{"type":"timesale","symbol":"SPY","exch":"Q","bid":"281.09","ask":"281.10","last":"281.10","size":"100","date":"1557757204760","seq":352342,"flag":"","cancel":false,"correction":false,"session":"normal"}"#;
    const TRADEX_FRAME: &str = r#"{"type":"tradex","symbol":"SPY","exch":"Q","price":"281.10","size":"100","cvol":"34507070","date":"1557757204760","last":"281.10"}"#;
    const MALFORMED_FRAME: &str = r#"{"type":"quote","symbol":"C","bid": not-json"#;

    /// Boots a market session wired to a leaked `SessionManager` and
    /// `Config` so the returned session satisfies `'static`. Leaking is
    /// acceptable here — each test is short-lived and owns its own
    /// `SessionManager`, so there is no cross-test contention.
    async fn build_market_session_against_port(port: u16) -> MarketSession<'static> {
        let server = Box::leak(Box::new(Server::new_async().await));
        let json_data = format!(
            r#"
        {{
            "stream": {{
                "url": "ws://127.0.0.1:{port}/v1/markets/events",
                "sessionid": "c8638963-a6d4-4fb9-9bc6-e25fbd8c60c3"
            }}
        }}
        "#
        );
        server
            .mock("POST", "/v1/markets/events/session")
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
        MarketSession::new_with_session_manager(config, session_manager)
            .await
            .expect("market session")
    }

    #[tokio::test]
    async fn test_market_event_stream_yields_typed_events_in_order() {
        let port = free_tcp_port();
        let session = build_market_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(QUOTE_FRAME),
                ScriptedWsAction::SendText(TRADE_FRAME),
                ScriptedWsAction::SendText(SUMMARY_FRAME),
                ScriptedWsAction::SendText(TIMESALE_FRAME),
                ScriptedWsAction::SendText(TRADEX_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_subscription_json| {},
        )
        .await;

        let symbols = ["SPY".to_string()];
        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .session_id(session.get_session_id())
            .build();

        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 5, "expected 5 events, got {collected:?}");
        assert!(matches!(collected[0], Ok(MarketEvent::Quote(_))));
        assert!(matches!(collected[1], Ok(MarketEvent::Trade(_))));
        assert!(matches!(collected[2], Ok(MarketEvent::Summary(_))));
        assert!(matches!(collected[3], Ok(MarketEvent::Timesale(_))));
        assert!(matches!(collected[4], Ok(MarketEvent::Tradex(_))));
    }

    #[tokio::test]
    async fn test_market_event_stream_malformed_frame_yields_decode_error_and_continues() {
        let port = free_tcp_port();
        let session = build_market_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(QUOTE_FRAME),
                ScriptedWsAction::SendText(MALFORMED_FRAME),
                ScriptedWsAction::SendText(TRADE_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let symbols = ["SPY".to_string()];
        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .session_id(session.get_session_id())
            .build();
        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 3, "expected 3 items, got {collected:?}");
        assert!(matches!(collected[0], Ok(MarketEvent::Quote(_))));
        assert!(matches!(collected[1], Err(Error::StreamDecodeError(_, _))));
        assert!(matches!(collected[2], Ok(MarketEvent::Trade(_))));
    }

    #[tokio::test]
    async fn test_market_event_stream_multiple_events_per_frame_decoded_individually() {
        let port = free_tcp_port();
        let session = build_market_session_against_port(port).await;

        // Two events on one line, separated by `\n`.
        let compound = Box::leak(format!("{}\n{}", QUOTE_FRAME, TRADE_FRAME).into_boxed_str());
        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(compound),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let symbols = ["SPY".to_string()];
        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .session_id(session.get_session_id())
            .build();
        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 2);
        assert!(matches!(collected[0], Ok(MarketEvent::Quote(_))));
        assert!(matches!(collected[1], Ok(MarketEvent::Trade(_))));
    }

    #[tokio::test]
    async fn test_market_event_stream_peer_close_terminates_without_error() {
        let port = free_tcp_port();
        let session = build_market_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendText(QUOTE_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let symbols = ["SPY".to_string()];
        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .session_id(session.get_session_id())
            .build();
        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 1);
        assert!(collected[0].is_ok());
    }

    #[tokio::test]
    async fn test_market_event_stream_ping_is_answered_and_does_not_yield_an_item() {
        let port = free_tcp_port();
        let session = build_market_session_against_port(port).await;

        scripted_websocket_server(
            ("127.0.0.1", port),
            vec![
                ScriptedWsAction::SendPing(b"hello"),
                ScriptedWsAction::SendText(QUOTE_FRAME),
                ScriptedWsAction::SendClose,
            ],
            |_sub| {},
        )
        .await;

        let symbols = ["SPY".to_string()];
        let payload = MarketSessionPayload::builder()
            .symbols(&symbols)
            .session_id(session.get_session_id())
            .build();
        let stream = session.event_stream(payload).await.expect("event_stream");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        // The ping does not surface as a user-visible item; only the
        // quote does.
        assert_eq!(collected.len(), 1);
        assert!(matches!(collected[0], Ok(MarketEvent::Quote(_))));
    }
}
