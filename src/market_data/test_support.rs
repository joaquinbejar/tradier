//! Test-only wire types for market-data fixtures.
//!
//! These `Serialize` / `proptest_derive::Arbitrary` structs mirror the
//! deserialized types in `types.rs` but go the other way: they produce JSON
//! we can feed back into the real deserializers and into JSON Schema
//! validators. They are gated to `cfg(test)` via the parent module.

use chrono::NaiveDate;
use proptest::{prelude::Strategy, strategy::Just};
use serde::Serialize;

/// A [`NaiveDate`] strategy that yields only valid wall-clock dates.
fn arb_naive_date() -> impl Strategy<Value = NaiveDate> {
    (1970i32..=2100, 1u32..=12, 1u32..=28)
        .prop_flat_map(|(y, m, d)| Just(NaiveDate::from_ymd_opt(y, m, d).unwrap()))
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct NaiveDateWire(#[proptest(strategy = "arb_naive_date()")] NaiveDate);

// -----------------------------------------------------------------------------
// Quotes wire types
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetQuotesResponseWire {
    pub quotes: QuotesPayloadWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct QuotesPayloadWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Vec<QuoteWire>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmatched_symbols: Option<UnmatchedSymbolsWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct UnmatchedSymbolsWire {
    pub symbol: Vec<String>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct QuoteWire {
    pub symbol: String,
    pub description: String,
    pub exch: String,
    #[serde(rename = "type")]
    pub quote_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_volume: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_volume: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prevclose: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week_52_high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week_52_low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bidsize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bidexch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asksize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub askexch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greeks: Option<GreeksDataWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GreeksDataWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamma: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theta: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vega: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rho: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phi: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_iv: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mid_iv: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_iv: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smv_vol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

// -----------------------------------------------------------------------------
// Option chains, strikes, expirations, lookup
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetOptionChainsResponseWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OptionChainsPayloadWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct OptionChainsPayloadWire {
    pub option: Vec<QuoteWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetOptionStrikesResponseWire {
    pub strikes: OptionStrikesPayloadWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct OptionStrikesPayloadWire {
    pub strike: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetOptionExpirationsResponseWire {
    pub expirations: OptionExpirationsPayloadWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct OptionExpirationsPayloadWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<Vec<ExpirationEntryWire>>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct ExpirationEntryWire {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikes: Option<ExpirationStrikesWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct ExpirationStrikesWire {
    pub strike: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct LookupOptionSymbolsResponseWire {
    pub symbols: Vec<LookupOptionSymbolsRootWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct LookupOptionSymbolsRootWire {
    #[serde(rename = "rootSymbol")]
    pub root_symbol: String,
    pub options: Vec<String>,
}

// -----------------------------------------------------------------------------
// History & timesales
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetHistoricalQuotesResponseWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryPayloadWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct HistoryPayloadWire {
    pub day: Vec<HistoryDayWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct HistoryDayWire {
    pub date: NaiveDateWire,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetTimeAndSalesResponseWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<TimeSalesPayloadWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct TimeSalesPayloadWire {
    pub data: Vec<TimeSalesEntryWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct TimeSalesEntryWire {
    pub time: String,
    pub timestamp: i64,
    pub price: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vwap: Option<f64>,
}

// -----------------------------------------------------------------------------
// ETB / Clock / Calendar
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetEtbSecuritiesResponseWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub securities: Option<EtbSecuritiesWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct EtbSecuritiesWire {
    pub security: Vec<EtbSecurityWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct EtbSecurityWire {
    pub symbol: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub security_type: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetClockResponseWire {
    pub clock: MarketClockWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct MarketClockWire {
    pub date: String,
    pub description: String,
    pub state: String,
    pub timestamp: i64,
    pub next_change: String,
    pub next_state: String,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct GetCalendarResponseWire {
    pub calendar: MarketCalendarWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct MarketCalendarWire {
    #[proptest(strategy = "1u32..=12")]
    pub month: u32,
    #[proptest(strategy = "1970u32..=2100")]
    pub year: u32,
    pub days: CalendarDaysWire,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct CalendarDaysWire {
    pub day: Vec<CalendarDayWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct CalendarDayWire {
    pub date: NaiveDateWire,
    pub status: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premarket: Option<CalendarWindowWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<CalendarWindowWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postmarket: Option<CalendarWindowWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct CalendarWindowWire {
    pub start: String,
    pub end: String,
}

// -----------------------------------------------------------------------------
// Search / Lookup
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct SearchCompaniesResponseWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub securities: Option<SearchSecuritiesWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct SearchSecuritiesWire {
    pub security: Vec<SearchSecurityWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct SearchSecurityWire {
    pub symbol: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub security_type: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct LookupSymbolResponseWire {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub securities: Option<LookupSecuritiesWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct LookupSecuritiesWire {
    pub security: Vec<LookupSecurityWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct LookupSecurityWire {
    pub symbol: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub security_type: String,
    pub description: String,
}

// -----------------------------------------------------------------------------
// Schema validation tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{Value, json};
    use std::fs::OpenOptions;

    static QUOTES_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_quotes_schema.json"
    );
    static CHAINS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_option_chains_schema.json"
    );
    static EXPIRATIONS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_option_expirations_schema.json"
    );
    static HISTORY_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_history_schema.json"
    );
    static TIMESALES_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_timesales_schema.json"
    );
    static CALENDAR_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_calendar_schema.json"
    );
    static LOOKUP_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/market_data/get_lookup_schema.json"
    );

    fn load_validator(path: &str) -> jsonschema::Validator {
        let reader = OpenOptions::new()
            .read(true)
            .open(path)
            .expect("schema file to exist");
        let reader = std::io::BufReader::new(reader);
        let schema: Value = serde_json::from_reader(reader).expect("schema JSON");
        jsonschema::validator_for(&schema).expect("validator")
    }

    #[test]
    fn empty_quotes_object_should_fail_schema_validation() {
        let validator = load_validator(QUOTES_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_chains_object_should_fail_schema_validation() {
        let validator = load_validator(CHAINS_PATH);
        assert!(!validator.is_valid(&json!({ "foo": 1 })));
    }

    #[test]
    fn empty_expirations_object_should_fail_schema_validation() {
        let validator = load_validator(EXPIRATIONS_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_history_object_should_fail_schema_validation() {
        let validator = load_validator(HISTORY_PATH);
        assert!(!validator.is_valid(&json!({ "foo": 1 })));
    }

    #[test]
    fn empty_timesales_object_should_fail_schema_validation() {
        let validator = load_validator(TIMESALES_PATH);
        assert!(!validator.is_valid(&json!({ "foo": 1 })));
    }

    #[test]
    fn empty_calendar_object_should_fail_schema_validation() {
        let validator = load_validator(CALENDAR_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_lookup_object_should_fail_schema_validation() {
        let validator = load_validator(LOOKUP_PATH);
        assert!(!validator.is_valid(&json!({ "foo": 1 })));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn serialized_quotes_wire_should_conform_to_schema(
            wire in any::<GetQuotesResponseWire>()
        ) {
            let validator = load_validator(QUOTES_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_chains_wire_should_conform_to_schema(
            wire in any::<GetOptionChainsResponseWire>()
        ) {
            let validator = load_validator(CHAINS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_expirations_wire_should_conform_to_schema(
            wire in any::<GetOptionExpirationsResponseWire>()
        ) {
            let validator = load_validator(EXPIRATIONS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_history_wire_should_conform_to_schema(
            wire in any::<GetHistoricalQuotesResponseWire>()
        ) {
            let validator = load_validator(HISTORY_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_timesales_wire_should_conform_to_schema(
            wire in any::<GetTimeAndSalesResponseWire>()
        ) {
            let validator = load_validator(TIMESALES_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_calendar_wire_should_conform_to_schema(
            wire in any::<GetCalendarResponseWire>()
        ) {
            let validator = load_validator(CALENDAR_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_lookup_wire_should_conform_to_schema(
            wire in any::<LookupSymbolResponseWire>()
        ) {
            let validator = load_validator(LOOKUP_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }
    }
}
