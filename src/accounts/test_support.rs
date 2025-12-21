use serde::Serialize;

use crate::{common::test_support::AccountTypeWire, utils::tests::DateTimeUtcWire};

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetAccountBalancesResponseWire {
    balances: GetAccountBalancesWire,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetAccountBalancesWire {
    option_short_value: f64,
    total_equity: f64,
    account_number: String,
    account_type: AccountTypeWire,
    close_pl: f64,
    current_requirement: f64,
    equity: f64,
    long_market_value: f64,
    market_value: f64,
    open_pl: f64,
    option_long_value: f64,
    option_requirement: f64,
    pending_orders_count: i32,
    short_market_value: f64,
    stock_long_value: f64,
    total_cash: f64,
    uncleared_funds: f64,
    pending_cash: f64,
    margin: MarginWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct MarginWire {
    fed_call: f64,
    maintenance_call: f64,
    option_buying_power: f64,
    stock_buying_power: f64,
    stock_short_value: f64,
    sweep: f64,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct PositionWire {
    cost_basis: f64,
    date_acquired: DateTimeUtcWire,
    id: u32,
    quantity: f64,
    symbol: String,
}
#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetAccountPositionsResponseWire {
    positions: Vec<PositionWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetAccountHistoryResponseWire {
    history: AccountHistoryEventsWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct AccountHistoryEventsWire {
    event: Vec<AccountEventWire>,
    #[proptest(strategy = "1..u32::MAX")]
    page: u32,
    #[proptest(strategy = "1..u32::MAX")]
    total_pages: u32,
    total_events: u32,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct AccountEventWire {
    date: DateTimeUtcWire,
    #[serde(rename = "type")]
    event_type: EventTypeWire,
    amount: f64,
    symbol: String,
    quantity: f64,
    price: f64,
    description: String,
    commission: f64,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EventTypeWire {
    Trade,
    Option,
    Ach,
    Wire,
    Dividend,
    Fee,
    Tax,
    Journal,
    Check,
    Transfer,
    Adjustment,
}

impl std::fmt::Display for EventTypeWire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            EventTypeWire::Trade => "trade",
            EventTypeWire::Option => "option",
            EventTypeWire::Ach => "ach",
            EventTypeWire::Wire => "wire",
            EventTypeWire::Dividend => "dividend",
            EventTypeWire::Fee => "fee",
            EventTypeWire::Tax => "tax",
            EventTypeWire::Journal => "journal",
            EventTypeWire::Check => "check",
            EventTypeWire::Transfer => "transfer",
            EventTypeWire::Adjustment => "adjustment",
        };
        f.write_str(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;

    static PATH: &str = "src/accounts/get_account_history_schema.json";

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
        fn serialized_wire_objects_should_conform_to_schema(wire_object in any::<GetAccountHistoryResponseWire>()) {
            let reader = OpenOptions::new().read(true).open(PATH).expect("schema file to exist and be readable");
            let reader = std::io::BufReader::new(reader);
            let schema: Value = serde_json::from_reader(reader)
                .expect("parsing the schema as a Value object to work");
            let validator = jsonschema::validator_for(&schema)
                .expect("validator in test to work as expected");
            let actual_serialized_value = serde_json::to_value(&wire_object)
                .expect("serde to serialize the object correctly");
            prop_assert!(validator.is_valid(&actual_serialized_value));
        }
    }
}
