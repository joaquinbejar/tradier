use std::error::Error;
use tracing::{error, info};
use tradier::config::Config;
use tradier::utils::logger::setup_logger;
use tradier::wssession::AccountSession;
use tradier::wssession::{MarketSession, MarketSessionFilter, MarketSessionPayload};

/// Initializes the program entry point.
///
/// - Sets up the logger using `setup_logger` for structured logging output.
/// - Loads API configuration from `Config`.
/// - Creates a `SessionManager` to manage singleton WebSocket sessions, enforcing that only one session
///   exists at a time.
///
/// The program enters an infinite loop to continuously create and manage WebSocket streaming sessions.
/// For each loop iteration:
///
/// 1. **MarketSession**: Attempts to create a market session for streaming real-time quotes and trades.
///    If successful, it streams data for specified symbols using `MarketSessionPayload`.
///    On any streaming error, it logs the issue and attempts to reconnect.
/// 2. **AccountSession**: Creates an account session to manage account-level WebSocket interactions,
///    but does not start streaming in this example. The account session creation is logged.
///
/// If either session fails to be created, an error message is logged, and the loop retries
/// after a 5-second delay.
///
/// ## Errors
/// Returns `Err` if there are any issues with initial setup, such as configuration loading or
/// logging initialization.
///
/// ## Returns
/// `Ok(())` if the main function completes successfully (though in this example, it will loop indefinitely).
///
/// ## Examples
/// ```no_run
/// tokio::spawn(async move {
///     main().await.expect("Main function failed");
/// });
/// ```
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
