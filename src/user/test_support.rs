use chrono::{DateTime, Utc};
use proptest::prelude::Strategy;
use serde::Serialize;

use crate::common::test_support::AccountTypeWire;

/// This is a class that's used to model the over-the-wire response of the GetUserProfile API
/// operation. This is used to generate valid JSON to use for testing deserialization of data
/// over the wire.
#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetUserProfileResponseWire {
    profile: UserProfileWire,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct UserProfileWire {
    id: String,
    name: String,
    account: Vec<AccountWire>,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct AccountWire {
    account_number: String,
    classification: ClassificationWire,
    #[serde(rename = "date_created")]
    date_created: DateTimeUtcWire,
    day_trader: bool,
    option_level: u8,
    status: AccountStatusWire,
    #[serde(rename = "type")]
    account_type: AccountTypeWire,
    #[serde(rename = "last_update_date")]
    last_update_date: DateTimeUtcWire,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "lowercase")]
pub enum ClassificationWire {
    Individual,
    Corporate,
    Joint,
    Ira,
    RothIra,
    Entity,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct DateTimeUtcWire(#[proptest(strategy = "arb_date_time_strategy()")] DateTime<Utc>);

/// This function creates arbitrary [chrono::DateTime<Utc>] ojbects.
///
/// Because DateTime itself already validates the input seconds and nanoseconds at runtime,
/// we limit the sample size of inputs to only valid ones.
fn arb_date_time_strategy() -> impl Strategy<Value = DateTime<Utc>> {
    (0..(i32::MAX as i64), ..=1_000_000_000u32).prop_filter_map(
        "Invalid DateTime objects are created as None.",
        |(seconds, nanos)| DateTime::from_timestamp(seconds, nanos),
    )
}
#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatusWire {
    Active,
    Closed,
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;
    use tracing::debug;

    static PATH: &str = "src/user/get_user_profile_schema.json";

    #[test]
    fn should_fail_to_process_an_empty_object() {
        let reader = OpenOptions::new()
            .read(true)
            .open(PATH)
            .expect("schema file to exist and be readable");
        let reader = std::io::BufReader::new(reader);
        let schema: Value =
            serde_json::from_reader(reader).expect("parsing the schema as a Value object to work");
        let validator =
            jsonschema::validator_for(&schema).expect("validator in test to work as expected");
        assert!(!validator.is_valid(
            &serde_json::to_value(json!({})).expect("serde to serialize the object correctly")
        ));
    }
    proptest! {

        #[test]
        fn serialized_wire_objects_should_conform_to_schema(wire_object in any::<GetUserProfileResponseWire>()) {
            let reader = OpenOptions::new().read(true).open(PATH).expect("schema file to exist and be readable");
            let reader = std::io::BufReader::new(reader);
            let schema: Value = serde_json::from_reader(reader)
                .expect("parsing the schema as a Value object to work");
            let validator = jsonschema::validator_for(&schema)
                .expect("validator in test to work as expected");
            let actual_serialized_value = serde_json::to_value(&wire_object)
                .expect("serde to serialize the object correctly");
            debug!("{:#?}", &actual_serialized_value);
            prop_assert!(validator.is_valid(&actual_serialized_value));
        }
    }
}
