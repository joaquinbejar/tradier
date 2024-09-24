use crate::config::base::Config;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketSessionFilter {
    TRADE,
    QUOTE,
    SUMMARY,
    TIMESALE,
    TRADEX,
}

impl MarketSessionFilter {
    fn as_str(&self) -> &'static str {
        match self {
            MarketSessionFilter::TRADE => "trade",
            MarketSessionFilter::QUOTE => "quote",
            MarketSessionFilter::SUMMARY => "summary",
            MarketSessionFilter::TIMESALE => "timesale",
            MarketSessionFilter::TRADEX => "tradex",
        }
    }
}

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

impl MarketSessionPayload {
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

    pub fn get_message(&self) -> Result<Message, Box<dyn Error>> {
        let result_payload_json = serde_json::to_value(self);
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let filter_str: Vec<String> = self.filter.as_ref().map_or(vec![], |filters| {
            filters.iter().map(|f| f.as_str().to_string()).collect()
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

pub struct MarketSession(Session);

impl MarketSession {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        match Session::new(SessionType::Market, config).await {
            Ok(session) => Ok(MarketSession(session)),
            Err(e) => Err(format!("Error creating market WS session: {}", e).into()),
        }
    }

    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }

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
