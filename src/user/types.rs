use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;

use crate::common::AccountType;
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
    pub account_number: String,
    pub classification: AccountClassification,
    pub date_created: DateTime<Utc>,
    pub day_trader: bool,
    pub option_level: u8,
    pub status: AccountStatus,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub last_update_date: DateTime<Utc>,
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

#[cfg(test)]
mod test {
    use proptest::{prelude::any, proptest};
    use tracing::debug;

    use crate::user::{test_support::GetUserProfileResponseWire, UserProfileResponse};
    proptest! {
        #[test]
        fn test_deserialize_user_profile_response_from_json(response in any::<GetUserProfileResponseWire>()) {

            let response = serde_json::to_string_pretty(&response)
                .expect("test fixture to serialize");
            let result: Result<UserProfileResponse, serde_json::Error> = serde_json::from_str(&response);
            debug!("{:#?}", result);
            assert!(result.is_ok());
        }
    }
}
