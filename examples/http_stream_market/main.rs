//! # HTTP streaming example
//!
//! Demonstrates how to consume Tradier market events over the HTTP
//! chunked-transfer fallback under
//! [`tradier::streaming::http_stream`]. The helpers reuse the pooled
//! `reqwest::Client` on the existing REST client.
//!
//! Reconnection is the caller's responsibility — the library surfaces
//! typed errors and callers drive the retry loop. This example logs
//! the first couple of events and exits.
//!
//! This example will only run end-to-end against a real Tradier
//! account; `cargo run --example http_stream_market` needs a valid
//! `TRADIER_ACCESS_TOKEN`. `cargo build --examples` compiles it
//! without credentials.

use futures_util::StreamExt;
use tracing::{error, info};
use tradier::Config;
use tradier::non_blocking::Client;
use tradier::streaming::http_stream;
use tradier::utils::logger::setup_logger;
use tradier::wssession::{MarketSession, MarketSessionFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    setup_logger();

    let config = Config::new();
    let rest = Client::new(config.clone());

    // Bootstrap a streaming session id (this is the same REST call the
    // WebSocket flavor uses — HTTP streaming and WS streaming share
    // session ids).
    let session = MarketSession::new(&config).await?;
    info!("HTTP streaming session bootstrapped");

    let symbols = ["AAPL".to_string(), "MSFT".to_string()];
    let filters = [MarketSessionFilter::QUOTE, MarketSessionFilter::TRADE];

    let events = http_stream::market_events(
        &rest,
        session.get_session_id(),
        &symbols,
        Some(&filters),
        Some(true),
        Some(true),
        None,
    )
    .await?;
    futures_util::pin_mut!(events);

    let mut printed = 0usize;
    while let Some(event) = events.next().await {
        match event {
            Ok(ev) => {
                info!(symbol = ev.symbol(), "market event");
                printed += 1;
                if printed >= 2 {
                    break;
                }
            }
            Err(e) => {
                error!(error = %e, "stream error");
                break;
            }
        }
    }

    Ok(())
}
