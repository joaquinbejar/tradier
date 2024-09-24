use std::error::Error;
use tokio;
use tracing::{error, info};
use tradier::utils::logger::setup_logger;
use tradier::config::base::Config;
use tradier::auth::oauth::OAuthClient;
use reqwest::Client;

async fn verify_authentication(access_token: &str, api_base_url: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client.get(&format!("{}/v1/user/profile", api_base_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        info!("Authentication successful. Response: {:?}", response.text().await?);
        Ok(())
    } else {
        Err(format!("Authentication failed. Status: {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let config = Config::new();
    info!("Config: {:?}", config);

    let access_token = config.credentials.access_token.clone()
        .expect("Access token not found in configuration");

    match verify_authentication(&access_token, &config.rest_api.base_url).await {
        Ok(_) => info!("Authentication verified successfully"),
        Err(e) => error!("Failed to verify authentication: {}", e),
    }

    Ok(())
}