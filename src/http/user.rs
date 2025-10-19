use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;

use crate::utils::OneOrMany;

#[derive(Debug, Deserialize, PartialEq)]
pub struct UserProfileResponse {
    pub profile: UserProfile,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub account: OneOrMany<Account>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Account {
    account_number: String,
    classification: AccountClassification,
    date_created: DateTime<Utc>,
    day_trader: bool,
    option_level: u8,
    status: AccountStatus,
    #[serde(rename = "type")]
    account_type: AccountType,
    last_update_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AccountClassification {
    Individual,
    Corporate,
    Joint,
    Ira,
    RothIra,
    Entity,
}

#[derive(Debug, Deserialize, PartialEq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Closed,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AccountType {
    Cash,
    Margin,
}

#[cfg(test)]
mod test {
    use crate::test_support::*;
    use proptest::prelude::*;

    use crate::http::user::UserProfileResponse;

    proptest! {
        #[test]
        fn test_deserialize_user_profile_response_from_json(response in any::<GetUserProfileResponseWire>()) {

            let response = serde_json::to_string_pretty(&response)
                .expect("test fixture to serialize");
            let result: Result<UserProfileResponse, serde_json::Error> = serde_json::from_str(&response);
            let result = result.inspect_err(|e| println!("{:?}", e));
            assert!(result.is_ok());
        }
    }
}
