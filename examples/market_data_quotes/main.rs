//! Example demonstrating `get_quotes` against the Tradier Market Data API.
//!
//! Run with:
//!
//! ```text
//! TRADIER_ACCESS_TOKEN=... \
//! cargo run --example market_data_quotes
//! ```
//!
//! Set `TRADIER_REST_BASE_URL` to `https://sandbox.tradier.com` to point at
//! the sandbox instead of production.
use tracing::info;
use tradier::Config;
use tradier::non_blocking::Client;
use tradier::non_blocking::operation::MarketData;
use tradier::types::{Greeks, Symbol, Symbols};
use tradier::utils::logger::setup_logger;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn core::error::Error>> {
    setup_logger();

    let config = Config::new();
    info!("Config: {:?}", &config);

    let client = Client::new(config);

    let symbols = Symbols::new([
        "AAPL".parse::<Symbol>()?,
        "MSFT".parse::<Symbol>()?,
        "SPY".parse::<Symbol>()?,
    ]);

    // Fetch quotes including Greeks (Greeks only apply to option quotes).
    let response = client.get_quotes(&symbols, Some(Greeks::new(true))).await?;

    info!("Quotes response: {:#?}", response);

    Ok(())
}
