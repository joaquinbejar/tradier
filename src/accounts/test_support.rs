use serde::Serialize;

use crate::{common::test_support::AccountTypeWire, utils::tests::DateTimeUtcWire};

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "snake_case")]
pub enum OrderTypeWire {
    Market,
    Limit,
    Stop,
    StopLimit,
    Debit,
    Credit,
    Even,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "snake_case")]
pub enum OrderSideWire {
    Buy,
    BuyToCover,
    Sell,
    SellShort,
    BuyToOpen,
    BuyToClose,
    SellToOpen,
    SellToClose,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatusWire {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Expired,
    Canceled,
    Rejected,
    PendingCancel,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "lowercase")]
pub enum OrderDurationWire {
    Day,
    Gtc,
    Pre,
    Post,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(rename_all = "lowercase")]
pub enum OrderClassWire {
    Equity,
    Option,
    Multileg,
    Combo,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct OrderLegWire {
    id: u32,
    #[serde(rename = "type")]
    order_type: OrderTypeWire,
    symbol: String,
    side: OrderSideWire,
    quantity: f64,
    status: OrderStatusWire,
    duration: OrderDurationWire,
    avg_fill_price: f64,
    exec_quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_fill_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_fill_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remaining_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    option_symbol: Option<String>,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct OrderWire {
    id: u32,
    #[serde(rename = "type")]
    order_type: OrderTypeWire,
    symbol: String,
    side: OrderSideWire,
    quantity: f64,
    status: OrderStatusWire,
    duration: OrderDurationWire,
    avg_fill_price: f64,
    exec_quantity: f64,
    create_date: DateTimeUtcWire,
    transaction_date: DateTimeUtcWire,
    class: OrderClassWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_fill_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_fill_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remaining_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    option_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_legs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    leg: Option<Vec<OrderLegWire>>,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct AccountOrdersWire {
    order: Vec<OrderWire>,
    #[proptest(strategy = "1..u32::MAX")]
    page: u32,
    #[proptest(strategy = "1..u32::MAX")]
    total_pages: u32,
    total_orders: u32,
}

#[derive(Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetAccountOrdersResponseWire {
    orders: AccountOrdersWire,
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;

    static PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/accounts/get_account_orders_schema.json"
    );

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

    // Reduced case count: OrderWire is deeply nested and the default 256 cases
    // exceed tarpaulin's timeout budget under instrumentation.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn serialized_wire_objects_should_conform_to_schema(wire_object in any::<GetAccountOrdersResponseWire>()) {
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
