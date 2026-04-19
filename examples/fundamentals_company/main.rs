//! Example demonstrating the Tradier Fundamentals (beta) REST endpoints.
//!
//! Run with:
//!
//! ```text
//! TRADIER_ACCESS_TOKEN=... \
//! cargo run --example fundamentals_company
//! ```
//!
//! Set `TRADIER_REST_BASE_URL` to `https://sandbox.tradier.com` to point at
//! the sandbox instead of production.

use tracing::info;
use tradier::non_blocking::operation::Fundamentals;
use tradier::non_blocking::Client;
use tradier::types::Symbol;
use tradier::utils::logger::setup_logger;
use tradier::Config;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn core::error::Error>> {
    setup_logger();

    let config = Config::new();
    info!("Config: {:?}", &config);

    let client = Client::new(config);

    let symbols = ["AAPL".parse::<Symbol>()?, "MSFT".parse::<Symbol>()?];

    // Company profile and share-class info.
    let company = client.get_company(&symbols).await?;
    info!("Company response: {:#?}", company);

    // Dividend history.
    let dividends = client.get_dividends(&symbols).await?;
    info!("Dividends response: {:#?}", dividends);

    // Price statistics.
    let statistics = client.get_statistics(&symbols).await?;
    info!("Statistics response: {:#?}", statistics);

    Ok(())
}
