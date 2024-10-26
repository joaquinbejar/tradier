use std::error::Error;
use tracing::{error, info};
use tradier::config::Config;
use tradier::utils::logger::setup_logger;
use tradier::wssession::account::AccountSession;
use tradier::wssession::market::{MarketSession, MarketSessionFilter, MarketSessionPayload};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let config = Config::new();

    loop {
        match MarketSession::new(&config).await {
            Ok(market_session) => {
                info!(
                    "Market streaming wssession created with id: {}",
                    market_session.get_session_id()
                );
                let payload = MarketSessionPayload {
                    symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
                    filter: Some(vec![MarketSessionFilter::QUOTE, MarketSessionFilter::TRADE]),
                    session_id: market_session.get_session_id().to_string(),
                    linebreak: Some(true),
                    valid_only: Some(true),
                    advanced_details: None,
                };
                if let Err(e) = market_session.ws_stream(payload).await {
                    error!("Streaming error: {}. Reconnecting...", e);
                }
            }
            Err(e) => {
                error!(
                    "Failed to create market streaming wssession: {}. Retrying...",
                    e
                );
            }
        }

        // Example of creating an AccountSession (not used in streaming)
        match AccountSession::new(&config).await {
            Ok(account_session) => {
                info!(
                    "Account wssession created with id: {}",
                    account_session.get_session_id()
                );
                // You can use account_session for account-related operations here
            }
            Err(e) => {
                error!("Failed to create account wssession: {}.", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
