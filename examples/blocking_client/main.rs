use tracing::info;
use tradier::blocking::{operation::Accounts, operation::User, Client};
use tradier::utils::logger::setup_logger;
use tradier::Config;

fn main() -> std::result::Result<(), Box<dyn core::error::Error>> {
    setup_logger();

    // Create a configuraton object
    let config = Config::new();
    info!("Config: {:?}", &config);

    let client = Client::new(config)?;

    let response = client.get_user_profile()?;
    println!("User Profile: {response:#?}");
    let account_number = match response.profile.account {
        tradier::types::OneOrMany::One(ref account) => &account.account_number,
        tradier::types::OneOrMany::Many(ref accounts) => {
            &accounts
                .first()
                .expect("at least one account")
                .account_number
        }
    }
    .parse()
    .expect("a valid account_number");
    let balances = client.get_account_balances(&account_number);

    println!("Account Balances: {balances:#?}");

    Ok(())
}
