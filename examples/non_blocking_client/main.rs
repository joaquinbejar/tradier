use tracing::info;
use tradier::non_blocking::operation::{Accounts, User};
use tradier::non_blocking::Client;
use tradier::types::AccountNumber;
use tradier::utils::logger::setup_logger;
use tradier::Config;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn core::error::Error>> {
    setup_logger();

    // Create a configuraton object
    let config = Config::new();
    info!("Config: {:?}", &config);

    let tradier_client = Client::new(config);
    let user_profile_response = tradier_client.get_user_profile().await?;

    info!("User Profile: {:#?}", user_profile_response);

    let account_number = match user_profile_response.profile.account {
        tradier::utils::OneOrMany::One(ref account) => &account.account_number,
        tradier::utils::OneOrMany::Many(ref items) => {
            &items.first().expect("at least one account").account_number
        }
    };
    let account_number: AccountNumber = account_number
        .parse()
        .expect("returned account number to be valid");

    let balances = tradier_client.get_account_balances(&account_number).await?;

    info!("Account balances: {balances:#?}");

    let positions = tradier_client
        .get_account_positions(&account_number)
        .await?;

    info!("Account positions: {positions:#?}");
    Ok(())
}
