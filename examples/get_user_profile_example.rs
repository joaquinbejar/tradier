use tracing::info;
use tradier::http::TradierRestClient;
use tradier::utils::logger::setup_logger;
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn core::error::Error>> {
    setup_logger();

    // Create a configuraton object
    let config = tradier::config::Config::new();
    info!("Config: {:?}", &config);

    let tradier_client = TradierRestClient::new(config);
    let user_profile_response = tradier_client.get_user_profile().await?;

    info!("User Profile: {:#?}", user_profile_response);
    Ok(())
}
