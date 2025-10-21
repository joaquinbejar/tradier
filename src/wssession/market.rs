use crate::config::Config;
use crate::wssession::session::{Session, SessionType};
use crate::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use tokio_tungstenite::connect_async;
use tracing::{error, info};
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

    /// Initiates a WebSocket connection and streams data based on the provided payload.
    ///
    /// # Arguments
    /// - `payload`: A `MarketSessionPayload` specifying symbols and settings for the WebSocket session.
    ///
    /// # Returns
    /// - `Ok(())`: If the WebSocket connection was successfully managed.
    /// - `Err(Box<dyn Error>)`: If the WebSocket connection or data streaming fails.
    ///
    /// # Behavior
    /// - Connects to the WebSocket endpoint.
    /// - Sends the specified `MarketSessionPayload`.
    /// - Listens for incoming messages and logs text or binary data.
    /// - Terminates on connection close or error.
    pub async fn ws_stream(&self, payload: MarketSessionPayload<'a>) -> Result<()> {
        let uri = &self.0.stream_info.url;
        let url = Url::parse(uri)?;

        info!("Connecting to: {}", uri);
        let (ws_stream, _) = connect_async(url.as_str()).await.map_err(Box::new)?;
        let (mut write, mut read) = ws_stream.split();

        let message = payload.get_message()?;
        write.send(message).await.map_err(Box::new)?;
        info!("Sent payload: {}", payload);

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    info!("Received: {}", text);
                }
                Ok(Message::Binary(data)) => {
                    info!("Received binary data: {} bytes", data.len());
                    // Handle binary data as needed
                }
                Ok(Message::Close(frame)) => {
                    info!("Connection closed: {:?}", frame);
                    break;
                }
                Err(e) => {
                    error!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::tests::{create_test_config, mock_websocket_server},
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
}
