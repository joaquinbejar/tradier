use std::error::Error;
use tokio;
use tracing::{error, info};
use reqwest::Client;
use tokio_tungstenite::tungstenite::protocol::Message;
use serde_json::json;
use tokio_tungstenite::connect_async;
use url::Url;
use tradier::utils::logger::setup_logger;
use tradier::config::base::Config;
use tradier::wssession::market::MarketSession;
use tradier::wssession::account::AccountSession;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let config = Config::new();

    loop {
        match MarketSession::new(&config).await {
            Ok(market_session) => {
                info!("Market streaming wssession created with id: {}", market_session.get_session_id());
                if let Err(e) = market_session.ws_stream(&["AAPL", "SPY210331C00300000"]).await {
                    error!("Streaming error: {}. Reconnecting...", e);
                }
            }
            Err(e) => {
                error!("Failed to create market streaming wssession: {}. Retrying...", e);
            }
        }

        // Example of creating an AccountSession (not used in streaming)
        match AccountSession::new(&config).await {
            Ok(account_session) => {
                info!("Account wssession created with id: {}", account_session.get_session_id());
                // You can use account_session for account-related operations here
            }
            Err(e) => {
                error!("Failed to create account wssession: {}.", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}