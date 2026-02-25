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
pub struct ClosedPositionWire {
    close_date: DateTimeUtcWire,
    cost: f64,
    gain_loss: f64,
    gain_loss_percent: f64,
    open_date: DateTimeUtcWire,
    proceeds: f64,
    quantity: f64,
    symbol: String,
    term: i32,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct AccountGainLossWire {
    closed_position: Vec<ClosedPositionWire>,
    page: u32,
    total_pages: u32,
    total_positions: u32,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetAccountGainLossResponseWire {
    gainloss: AccountGainLossWire,
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;

    static GAINLOSS_PATH: &str = "src/accounts/get_account_gainloss_schema.json";

    #[test]
    fn empty_gainloss_object_should_fail_schema_validation() {
        let reader = OpenOptions::new()
            .read(true)
            .open(GAINLOSS_PATH)
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
        fn serialized_gainloss_wire_objects_should_conform_to_schema(
            wire in any::<GetAccountGainLossResponseWire>()
        ) {
            let reader = OpenOptions::new()
                .read(true)
                .open(GAINLOSS_PATH)
                .expect("schema file to exist and be readable");
            let reader = std::io::BufReader::new(reader);
            let schema: Value = serde_json::from_reader(reader)
                .expect("parsing the schema as a Value object to work");
            let validator = jsonschema::validator_for(&schema)
                .expect("validator in test to work as expected");
            let actual_serialized_value = serde_json::to_value(&wire)
                .expect("serde to serialize the object correctly");
            prop_assert!(validator.is_valid(&actual_serialized_value));
        }
    }
}
