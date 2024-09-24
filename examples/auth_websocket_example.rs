use std::error::Error;
use tokio;
use tracing::{error, info, debug};
use reqwest::Client as HttpClient;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use url::Url;
use serde_json::json;
use tradier::utils::logger::setup_logger;
use tradier::config::base::Config;

async fn create_streaming_session(access_token: &str) -> Result<String, Box<dyn Error>> {
    let client = HttpClient::new();
    let url = "https://api.tradier.com/v1/markets/events/session";

    let response = client.post(url)
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
        let json_response: serde_json::Value = serde_json::from_str(&body)?;
        let session_id = json_response["stream"]["sessionid"].as_str()
            .ok_or("Session ID not found in response")?;
        Ok(session_id.to_string())
    } else {
        Err(format!("Failed to create session. Status: {}. Body: {}", status, body).into())
    }
}

async fn connect_and_stream(session_id: &str, symbols: &[&str]) -> Result<(), Box<dyn Error>> {
    let uri = "wss://ws.tradier.com/v1/markets/events";
    let url = Url::parse(uri)?;

    let (ws_stream, _) = connect_async(url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    let payload = json!({
        "symbols": symbols,
        "filter": ["quote"],
        "sessionid": session_id,
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
            Ok(Message::Close(frame)) => {
                info!("WebSocket closed: {:?}", frame);
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let config = Config::new();
    let access_token = config.credentials.access_token.clone()
        .expect("Access token not found in configuration");

    loop {
        match create_streaming_session(&access_token).await {
            Ok(session_id) => {
                info!("Streaming session created with id: {}", session_id);
                if let Err(e) = connect_and_stream(&session_id, &["AAPL", "MSFT"]).await {
                    error!("Streaming error: {}. Reconnecting...", e);
                }
            },
            Err(e) => {
                error!("Failed to create streaming session: {}. Retrying...", e);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}