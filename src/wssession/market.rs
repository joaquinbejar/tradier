use crate::config::Config;
use crate::constants::TRADIER_WS_BASE_URL;
use crate::wssession::session::{Session, SessionType};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use tokio_tungstenite::connect_async;
use tracing::{error, info};
use tungstenite::Message;
use url::Url;

/// `MarketSessionFilter` represents the possible filters for a market WebSocket session.
///
/// Options include:
/// - `TRADE`: Filters trade-related events.
/// - `QUOTE`: Filters quote-related events.
/// - `SUMMARY`: Filters summary events.
/// - `TIMESALE`: Filters time sale events.
/// - `TRADEX`: Filters extended trade events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(into = "String")]
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
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "trade" => Ok(MarketSessionFilter::TRADE),
            "quote" => Ok(MarketSessionFilter::QUOTE),
            "summary" => Ok(MarketSessionFilter::SUMMARY),
            "timesale" => Ok(MarketSessionFilter::TIMESALE),
            "tradex" => Ok(MarketSessionFilter::TRADEX),
            _ => Err(format!("Unsupported MarketSessionFilter: {}", value).into()),
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
/// - `linebreak`: Optional boolean to indicate if line breaks are to be used in streaming data.
/// - `valid_only`: Optional boolean that, if set to `true`, filters out invalid data.
/// - `advanced_details`: Optional boolean for additional data in advanced detail format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSessionPayload {
    pub symbols: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<MarketSessionFilter>>,
    #[serde(rename = "sessionid")]
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linebreak: Option<bool>,
    #[serde(rename = "validOnly", skip_serializing_if = "Option::is_none")]
    pub valid_only: Option<bool>,
    #[serde(rename = "advancedDetails", skip_serializing_if = "Option::is_none")]
    pub advanced_details: Option<bool>,
}

#[bon::bon]
impl MarketSessionPayload {
    /// Constructs a new `MarketSessionPayload` with default filter settings.
    ///
    /// # Arguments
    /// - `symbols`: A vector of symbols to subscribe to.
    /// - `session_id`: A unique session identifier for the WebSocket connection.
    ///
    /// # Returns
    /// - `Self`: A new `MarketSessionPayload` instance with default filter and optional settings.
    #[builder]
    pub fn new(symbols: Vec<String>, session_id: String) -> Self {
        MarketSessionPayload {
            symbols,
            filter: Some(vec![MarketSessionFilter::QUOTE]),
            session_id,
            linebreak: Some(true),
            valid_only: Some(false),
            advanced_details: Some(false),
        }
    }

    /// Converts the payload to a WebSocket `Message` for sending.
    ///
    /// # Returns
    /// - `Ok(Message)`: The WebSocket message if serialization is successful.
    /// - `Err(Box<dyn Error>)`: An error if serialization fails.
    pub fn get_message(&self) -> Result<Message, Box<dyn Error>> {
        let result_payload_json = dbg!(serde_json::to_string(self));
        match result_payload_json {
            Ok(value) => Ok(Message::Text(value.to_string())),
            Err(e) => {
                error!("Error parsing message");
                Err(e.into())
            }
        }
    }
}

impl Display for MarketSessionPayload {
    /// Implements `Display` for `MarketSessionPayload` to show formatted details.
    ///
    /// Format includes:
    /// - Symbols
    /// - Filters
    /// - Session ID
    /// - Optional settings (linebreak, valid_only, advanced_details)
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let filter_str: Vec<String> = self.filter.as_ref().map_or(vec![], |filters| {
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
pub struct MarketSession(Session);

impl MarketSession {
    /// Creates a new `MarketSession` using the specified configuration.
    ///
    /// # Arguments
    /// - `config`: A reference to the `Config` structure containing API settings.
    ///
    /// # Returns
    /// - `Ok(MarketSession)`: A `MarketSession` instance if the session was created successfully.
    /// - `Err(Box<dyn Error>)`: If session creation fails.
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        match Session::new(SessionType::Market, config).await {
            Ok(session) => Ok(MarketSession(session)),
            Err(e) => Err(format!("Error creating market WS session: {}", e).into()),
        }
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
    pub async fn ws_stream(&self, payload: MarketSessionPayload) -> Result<(), Box<dyn Error>> {
        let uri = &format!("{}/v1/markets/events", TRADIER_WS_BASE_URL);
        let url = Url::parse(uri)?;

        info!("Connecting to: {}", uri);
        let (ws_stream, _) = connect_async(url.as_str()).await?;
        let (mut write, mut read) = ws_stream.split();

        let message = payload.get_message()?;
        write.send(message).await?;
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
    use super::*;
    use tungstenite::Message;

    #[test]
    fn test_market_session_filter_as_ref() {
        assert_eq!(MarketSessionFilter::TRADE.as_ref(), "trade");
        assert_eq!(MarketSessionFilter::QUOTE.as_ref(), "quote");
        assert_eq!(MarketSessionFilter::SUMMARY.as_ref(), "summary");
        assert_eq!(MarketSessionFilter::TIMESALE.as_ref(), "timesale");
        assert_eq!(MarketSessionFilter::TRADEX.as_ref(), "tradex");
    }

    #[test]
    fn test_market_session_payload_new() {
        let symbols = vec!["AAPL".to_string(), "TSLA".to_string()];
        let session_id = "test-session-id".to_string();

        let payload = MarketSessionPayload::builder()
            .session_id(session_id.clone())
            .symbols(symbols.clone())
            .build();

        assert_eq!(payload.symbols, symbols);
        assert_eq!(payload.filter, Some(vec![MarketSessionFilter::QUOTE]));
        assert_eq!(payload.session_id, session_id);
        assert_eq!(payload.linebreak, Some(true));
        assert_eq!(payload.valid_only, Some(false));
        assert_eq!(payload.advanced_details, Some(false));
    }

    #[test]
    fn test_market_session_payload_get_message() {
        let payload = MarketSessionPayload {
            symbols: vec!["AAPL".to_string()],
            filter: Some(vec![MarketSessionFilter::TRADE]),
            session_id: "test-session-id".to_string(),
            linebreak: Some(true),
            valid_only: Some(true),
            advanced_details: Some(false),
        };

        let message_result = payload.get_message();
        assert!(message_result.is_ok());

        let message = message_result.unwrap();
        if let Message::Text(text) = message {
            assert!(text.contains("\"symbols\":[\"AAPL\"]"));
            assert!(text.contains("\"filter\":[\"trade\"]"));
            assert!(text.contains("\"sessionid\":\"test-session-id\""));
            assert!(text.contains("\"linebreak\":true"));
            assert!(text.contains("\"validOnly\":true"));
            assert!(text.contains("\"advancedDetails\":false"));
        } else {
            panic!("Expected a text message");
        }
    }

    #[test]
    fn test_market_session_payload_display() {
        let payload = MarketSessionPayload {
            symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
            filter: Some(vec![MarketSessionFilter::TRADE, MarketSessionFilter::QUOTE]),
            session_id: "display-session".to_string(),
            linebreak: None,
            valid_only: None,
            advanced_details: Some(true),
        };

        let display_output = format!("{}", payload);
        assert!(display_output.contains("symbols: [\"AAPL\", \"MSFT\"]"));
        assert!(display_output.contains("filter: [\"trade\", \"quote\"]"));
        assert!(display_output.contains("session_id: display-session"));
        assert!(display_output.contains("linebreak: None"));
        assert!(display_output.contains("valid_only: None"));
        assert!(display_output.contains("advanced_details: true"));
    }
}
