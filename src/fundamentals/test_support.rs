//! Test-only wire types for fundamentals fixtures.
//!
//! These `Serialize` / `proptest_derive::Arbitrary` structs produce JSON we
//! can feed back into the real deserializers and into JSON Schema
//! validators.
//!
//! # Design note
//!
//! The real domain types in `types.rs` are deeply nested (envelope ->
//! result -> tables -> table -> row). If we derive `Arbitrary` straight
//! through that tree, the strategy tree itself blows the default Rust
//! test-thread stack. Here we flatten the Wire types one level and leave
//! most of the sub-tables out of the fuzzed payload; the exact parsing
//! of each sub-shape is covered by hand-written fixture tests in
//! `types.rs` instead.

use serde::Serialize;

/// Short ASCII string wrapper used in fuzzed fixtures to bound payload size.
#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct ShortString(
    #[proptest(regex = r"[A-Za-z0-9 _\\-]{0,8}")] //
    String,
);

// -----------------------------------------------------------------------------
// Flat envelope shared by every endpoint
// -----------------------------------------------------------------------------

/// A minimal flat result: preserves the `type` discriminator and the optional
/// `id`, but leaves `tables` as an empty object. The deserializer's treatment
/// of the surrounding envelope is what we actually fuzz; per-table shapes are
/// exercised by hand-written fixture tests in `types.rs`.
#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct FlatResultWire {
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ShortString>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct CompanyResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct CompanyResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<CompanyResponseWire>(), 0..=2)"
    )]
    pub Vec<CompanyResponseWire>,
);

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct CorporateCalendarResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct CorporateCalendarResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<CorporateCalendarResponseWire>(), 0..=2)"
    )]
    pub Vec<CorporateCalendarResponseWire>,
);

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct DividendResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct DividendResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<DividendResponseWire>(), 0..=2)"
    )]
    pub Vec<DividendResponseWire>,
);

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct CorporateActionResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct CorporateActionResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<CorporateActionResponseWire>(), 0..=2)"
    )]
    pub Vec<CorporateActionResponseWire>,
);

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct RatiosResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct RatiosResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<RatiosResponseWire>(), 0..=2)"
    )]
    pub Vec<RatiosResponseWire>,
);

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct FinancialsResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct FinancialsResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FinancialsResponseWire>(), 0..=2)"
    )]
    pub Vec<FinancialsResponseWire>,
);

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
pub struct StatisticsResponseWire {
    pub request: ShortString,
    #[serde(rename = "type")]
    pub kind: ShortString,
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<FlatResultWire>(), 0..=2)"
    )]
    pub results: Vec<FlatResultWire>,
}

#[derive(Clone, Debug, Serialize, proptest_derive::Arbitrary)]
#[serde(transparent)]
pub struct StatisticsResponseArrayWire(
    #[proptest(
        strategy = "proptest::collection::vec(proptest::prelude::any::<StatisticsResponseWire>(), 0..=2)"
    )]
    pub Vec<StatisticsResponseWire>,
);

// -----------------------------------------------------------------------------
// Schema validation tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;

    static COMPANY_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_company_schema.json"
    );
    static CORP_CAL_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_corporate_calendars_schema.json"
    );
    static DIVIDENDS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_dividends_schema.json"
    );
    static CORP_ACTIONS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_corporate_actions_schema.json"
    );
    static RATIOS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_ratios_schema.json"
    );
    static FINANCIALS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_financials_schema.json"
    );
    static STATISTICS_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/fundamentals/get_statistics_schema.json"
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
    fn empty_object_should_fail_company_schema() {
        let validator = load_validator(COMPANY_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_object_should_fail_corporate_calendars_schema() {
        let validator = load_validator(CORP_CAL_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_object_should_fail_dividends_schema() {
        let validator = load_validator(DIVIDENDS_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_object_should_fail_corporate_actions_schema() {
        let validator = load_validator(CORP_ACTIONS_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_object_should_fail_ratios_schema() {
        let validator = load_validator(RATIOS_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_object_should_fail_financials_schema() {
        let validator = load_validator(FINANCIALS_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_object_should_fail_statistics_schema() {
        let validator = load_validator(STATISTICS_PATH);
        assert!(!validator.is_valid(&json!({})));
    }

    #[test]
    fn empty_array_should_validate_against_all_schemas() {
        for p in [
            COMPANY_PATH,
            CORP_CAL_PATH,
            DIVIDENDS_PATH,
            CORP_ACTIONS_PATH,
            RATIOS_PATH,
            FINANCIALS_PATH,
            STATISTICS_PATH,
        ] {
            let validator = load_validator(p);
            assert!(validator.is_valid(&json!([])), "schema {} rejected []", p);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn serialized_company_wire_should_conform_to_schema(
            wire in any::<CompanyResponseArrayWire>()
        ) {
            let validator = load_validator(COMPANY_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_corporate_calendar_wire_should_conform_to_schema(
            wire in any::<CorporateCalendarResponseArrayWire>()
        ) {
            let validator = load_validator(CORP_CAL_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_dividend_wire_should_conform_to_schema(
            wire in any::<DividendResponseArrayWire>()
        ) {
            let validator = load_validator(DIVIDENDS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_corporate_action_wire_should_conform_to_schema(
            wire in any::<CorporateActionResponseArrayWire>()
        ) {
            let validator = load_validator(CORP_ACTIONS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_ratios_wire_should_conform_to_schema(
            wire in any::<RatiosResponseArrayWire>()
        ) {
            let validator = load_validator(RATIOS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_financials_wire_should_conform_to_schema(
            wire in any::<FinancialsResponseArrayWire>()
        ) {
            let validator = load_validator(FINANCIALS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }

        #[test]
        fn serialized_statistics_wire_should_conform_to_schema(
            wire in any::<StatisticsResponseArrayWire>()
        ) {
            let validator = load_validator(STATISTICS_PATH);
            let value = serde_json::to_value(&wire).expect("serialize");
            prop_assert!(validator.is_valid(&value), "value: {}", value);
        }
    }
}
