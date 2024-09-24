use crate::config::base::Config;
use crate::wssession::session::{Session, SessionType};
use std::error::Error;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::connect_async;
use tracing::{error, info};
use tungstenite::Message;
use url::Url;
use reqwest::Client;
use crate::constants::TRADIER_WS_BASE_URL;

pub enum MarketSessionFilter {
    TRADE,
    QUOTE,
    SUMMARY,
    TIMESALE,
    TRADEX
}

pub struct MarketSession(Session);

impl MarketSession {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        match Session::new(SessionType::Market, config).await {
            Ok(session) => Ok(MarketSession(session)),
            Err(e) => {
                Err(format!("Error creating account wssession: {}", e).into())
            }
        }
    }

    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }

    pub async fn ws_stream(&self, symbols: &[&str]) -> Result<(), Box<dyn Error>> {
        let uri = &format!("{}/v1/markets/events", TRADIER_WS_BASE_URL);
        let url = Url::parse(uri)?;

        info!("Connecting to: {}", uri);
        let (ws_stream, _) = connect_async(url.as_str()).await?;
        let (mut write, mut read) = ws_stream.split();

        let payload = json!({
        "symbols": symbols,
        "filter": ["quote"],
        "sessionid": self.get_session_id(),
        "linebreak": true,
        "validOnly": false
    });

        write.send(Message::Text(payload.to_string())).await?;
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

