//! Request and response types for the Tradier Fundamentals (beta) REST endpoints.
//!
//! Upstream documentation:
//! <https://documentation.tradier.com/brokerage-api/markets/get-company>
//!
//! # Wire shape
//!
//! Every fundamentals endpoint returns a JSON array at the top level. Each
//! element wraps the data for one requested symbol:
//!
//! ```json
//! [
//!   {
//!     "request": "AAPL",
//!     "type": "Symbol",
//!     "results": [
//!       { "type": "Company", "id": "...", "tables": { ... } },
//!       ...
//!     ]
//!   }
//! ]
//! ```
//!
//! The `tables` sub-object varies per endpoint (company details, dividends,
//! ratios, etc.). To stay faithful to the wire while tolerating the heavy
//! optional-field surface, many leaf fields are kept as `Option<String>` /
//! `Option<f64>` instead of stronger domain types — mirror the wire first,
//! add typed helpers on top only where the domain clearly benefits.

use chrono::NaiveDate;
use serde::Deserialize;

use crate::utils::OneOrMany;

pub use crate::common::Symbol;

// -----------------------------------------------------------------------------
// Shared request envelope
// -----------------------------------------------------------------------------

/// A single top-level entry in a fundamentals response array. The `request`
/// field echoes the ticker the server was asked about; `results` holds the
/// endpoint-specific payload.
///
/// This type is generic over `T`, the endpoint-specific `results` inner type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FundamentalsEnvelope<T> {
    /// Echo of the ticker the server was asked about.
    pub request: String,
    /// Always `"Symbol"` in production responses.
    #[serde(rename = "type")]
    pub kind: String,
    /// Endpoint-specific payload. Tradier usually returns an array; some
    /// endpoints omit the field when no data is available.
    #[serde(default = "empty_results")]
    pub results: Vec<T>,
}

fn empty_results<T>() -> Vec<T> {
    Vec::new()
}

// -----------------------------------------------------------------------------
// 1. GET /beta/markets/fundamentals/company
// -----------------------------------------------------------------------------

/// One element of the response array for `GET /beta/markets/fundamentals/company`.
pub type CompanyResponse = FundamentalsEnvelope<CompanyResult>;

/// Individual result block inside a company response. Tradier returns both
/// `Company` profile blocks and `Stock` data blocks interleaved in a single
/// `results` array; we capture them uniformly and keep the raw `tables`
/// map for the caller to inspect.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CompanyResult {
    /// Discriminator: `"Company"`, `"Stock"`, `"IPO"`, `"Security"`, etc.
    #[serde(rename = "type")]
    pub kind: String,
    /// Tradier-internal identifier for this result row.
    #[serde(default)]
    pub id: Option<String>,
    /// Raw nested `tables` structure. Shape varies with `kind`; kept as a
    /// generic JSON value so we do not lose data upstream adds later.
    #[serde(default)]
    pub tables: Option<CompanyTables>,
}

/// Typed view over the common subset of company/stock `tables` entries.
///
/// Tradier returns a large number of optional fields, many of them as
/// strings even when numeric. To stay faithful to the wire we keep them
/// as `Option<String>` and expose typed helpers only where a caller
/// clearly benefits.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CompanyTables {
    #[serde(default)]
    pub company_profile: Option<CompanyProfile>,
    #[serde(default)]
    pub asset_classification: Option<AssetClassification>,
    #[serde(default)]
    pub historical_asset_classification: Option<AssetClassification>,
    #[serde(default)]
    pub long_descriptions: Option<String>,
    #[serde(default)]
    pub share_class_profile: Option<ShareClassProfile>,
    #[serde(default)]
    pub share_class: Option<ShareClass>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CompanyProfile {
    #[serde(default)]
    pub company_id: Option<String>,
    #[serde(default)]
    pub average_employee_number: Option<i64>,
    #[serde(default)]
    pub contact_email: Option<String>,
    #[serde(default)]
    pub headquarter: Option<Headquarter>,
    #[serde(default)]
    pub is_head_office_same_with_registered_office_flag: Option<bool>,
    #[serde(default)]
    pub total_employee_number: Option<i64>,
    #[serde(default)]
    pub total_employee_number_as_of_date: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Headquarter {
    #[serde(default)]
    pub address_line1: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub fax: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub province: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AssetClassification {
    #[serde(default)]
    pub financial_health_grade: Option<String>,
    #[serde(default)]
    pub growth_grade: Option<String>,
    #[serde(default)]
    pub morningstar_economy_sphere_code: Option<i64>,
    #[serde(default)]
    pub morningstar_industry_code: Option<i64>,
    #[serde(default)]
    pub morningstar_industry_group_code: Option<i64>,
    #[serde(default)]
    pub morningstar_sector_code: Option<i64>,
    #[serde(default)]
    pub nace: Option<String>,
    #[serde(default)]
    pub naics: Option<String>,
    #[serde(default)]
    pub profitability_grade: Option<String>,
    #[serde(default)]
    pub sic: Option<String>,
    #[serde(default)]
    pub stock_type: Option<i64>,
    #[serde(default)]
    pub style_box: Option<i64>,
    #[serde(default)]
    pub value_score: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ShareClassProfile {
    #[serde(default)]
    pub share_class_id: Option<String>,
    #[serde(default)]
    pub enterprise_value: Option<f64>,
    #[serde(default)]
    pub market_cap: Option<f64>,
    #[serde(default)]
    pub shares_outstanding: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ShareClass {
    #[serde(default)]
    pub currency_id: Option<String>,
    #[serde(default)]
    pub exchange_id: Option<String>,
    #[serde(default)]
    pub ipo_date: Option<String>,
    #[serde(default)]
    pub is_primary_share: Option<bool>,
    #[serde(default)]
    pub security_type: Option<String>,
    #[serde(default)]
    pub share_class_description: Option<String>,
    #[serde(default)]
    pub share_class_id: Option<String>,
    #[serde(default)]
    pub share_class_status: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub trading_status: Option<bool>,
}

// -----------------------------------------------------------------------------
// 2. GET /beta/markets/fundamentals/corporate_calendars
// -----------------------------------------------------------------------------

/// One element of the response array for
/// `GET /beta/markets/fundamentals/corporate_calendars`.
pub type CorporateCalendarResponse = FundamentalsEnvelope<CorporateCalendarResult>;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CorporateCalendarResult {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tables: Option<CorporateCalendarTables>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CorporateCalendarTables {
    #[serde(default)]
    pub corporate_calendars: Option<OneOrMany<CorporateCalendarEvent>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CorporateCalendarEvent {
    #[serde(default)]
    pub company_id: Option<String>,
    #[serde(default)]
    pub begin_date_time: Option<String>,
    #[serde(default)]
    pub end_date_time: Option<String>,
    #[serde(default)]
    pub event_type: Option<i64>,
    #[serde(default)]
    pub estimated_date_for_next_event: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub event_fiscal_year: Option<i64>,
    #[serde(default)]
    pub event_status: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
}

// -----------------------------------------------------------------------------
// 3. GET /beta/markets/fundamentals/dividends
// -----------------------------------------------------------------------------

/// One element of the response array for
/// `GET /beta/markets/fundamentals/dividends`.
pub type DividendResponse = FundamentalsEnvelope<DividendResult>;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DividendResult {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tables: Option<DividendTables>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct DividendTables {
    #[serde(default)]
    pub cash_dividends: Option<OneOrMany<CashDividend>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CashDividend {
    #[serde(default)]
    pub share_class_id: Option<String>,
    #[serde(default)]
    pub dividend_type: Option<String>,
    #[serde(default)]
    pub ex_date: Option<NaiveDate>,
    #[serde(default)]
    pub cash_amount: Option<f64>,
    #[serde(default)]
    pub currency_i_d: Option<String>,
    #[serde(default)]
    pub declaration_date: Option<NaiveDate>,
    #[serde(default)]
    pub frequency: Option<i64>,
    #[serde(default)]
    pub pay_date: Option<NaiveDate>,
    #[serde(default)]
    pub record_date: Option<NaiveDate>,
}

// -----------------------------------------------------------------------------
// 4. GET /beta/markets/fundamentals/corporate_actions
// -----------------------------------------------------------------------------

/// One element of the response array for
/// `GET /beta/markets/fundamentals/corporate_actions`.
pub type CorporateActionResponse = FundamentalsEnvelope<CorporateActionResult>;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CorporateActionResult {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tables: Option<CorporateActionTables>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CorporateActionTables {
    #[serde(default)]
    pub stock_splits: Option<OneOrMany<StockSplit>>,
    #[serde(default)]
    pub mergers_and_acquisitions: Option<OneOrMany<MergerAcquisition>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct StockSplit {
    #[serde(default)]
    pub share_class_id: Option<String>,
    #[serde(default)]
    pub ex_date: Option<NaiveDate>,
    #[serde(default)]
    pub adjustment_factor: Option<f64>,
    #[serde(default)]
    pub split_from: Option<f64>,
    #[serde(default)]
    pub split_to: Option<f64>,
    #[serde(default)]
    pub split_type: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct MergerAcquisition {
    #[serde(default)]
    pub acquired_company_id: Option<String>,
    #[serde(default)]
    pub parent_company_id: Option<String>,
    #[serde(default)]
    pub cash_amount: Option<f64>,
    #[serde(default)]
    pub currency_id: Option<String>,
    #[serde(default)]
    pub effective_date: Option<NaiveDate>,
    #[serde(default)]
    pub notes: Option<String>,
}

// -----------------------------------------------------------------------------
// 5. GET /beta/markets/fundamentals/ratios
// -----------------------------------------------------------------------------

/// One element of the response array for
/// `GET /beta/markets/fundamentals/ratios`.
pub type RatiosResponse = FundamentalsEnvelope<RatiosResult>;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RatiosResult {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tables: Option<RatiosTables>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct RatiosTables {
    #[serde(default)]
    pub operation_ratios_restate: Option<OneOrMany<OperationRatios>>,
    #[serde(default)]
    pub earning_ratios_restate: Option<OneOrMany<EarningRatios>>,
    #[serde(default)]
    pub valuation_ratios: Option<OneOrMany<ValuationRatios>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct OperationRatios {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub report_type: Option<String>,
    #[serde(default)]
    pub fiscal_year_end: Option<i64>,
    #[serde(default)]
    pub assets_turnover: Option<f64>,
    #[serde(default)]
    pub debt_to_assets: Option<f64>,
    #[serde(default)]
    pub gross_margin: Option<f64>,
    #[serde(default)]
    pub net_margin: Option<f64>,
    #[serde(default)]
    pub operation_margin: Option<f64>,
    #[serde(default)]
    pub quick_ratio: Option<f64>,
    #[serde(default)]
    pub r_o_a: Option<f64>,
    #[serde(default)]
    pub r_o_e: Option<f64>,
    #[serde(default)]
    pub r_o_i_c: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct EarningRatios {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub report_type: Option<String>,
    #[serde(default)]
    pub fiscal_year_end: Option<i64>,
    #[serde(default)]
    pub diluted_eps_growth: Option<f64>,
    #[serde(default)]
    pub d_p_s_growth: Option<f64>,
    #[serde(default)]
    pub equity_per_share_growth: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ValuationRatios {
    #[serde(default)]
    pub as_of_date: Option<NaiveDate>,
    #[serde(default)]
    pub dividend_yield: Option<f64>,
    #[serde(default)]
    pub earning_yield: Option<f64>,
    #[serde(default)]
    pub forward_dividend_yield: Option<f64>,
    #[serde(default)]
    pub forward_p_e_ratio: Option<f64>,
    #[serde(default)]
    pub payout_ratio: Option<f64>,
    #[serde(default)]
    pub p_b_ratio: Option<f64>,
    #[serde(default)]
    pub p_e_ratio: Option<f64>,
    #[serde(default)]
    pub peg_ratio: Option<f64>,
    #[serde(default)]
    pub p_s_ratio: Option<f64>,
}

// -----------------------------------------------------------------------------
// 6. GET /beta/markets/fundamentals/financials
// -----------------------------------------------------------------------------

/// One element of the response array for
/// `GET /beta/markets/fundamentals/financials`.
pub type FinancialsResponse = FundamentalsEnvelope<FinancialsResult>;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FinancialsResult {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tables: Option<FinancialsTables>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct FinancialsTables {
    #[serde(default)]
    pub balance_sheet: Option<OneOrMany<FinancialStatement>>,
    #[serde(default)]
    pub cash_flow_statement: Option<OneOrMany<FinancialStatement>>,
    #[serde(default)]
    pub income_statement: Option<OneOrMany<FinancialStatement>>,
}

/// A single financial statement row. Tradier ships every line-item as an
/// arbitrary number of optional `f64` / `String` fields, many of them sparse.
/// We capture the commonly-populated header fields and preserve the rest
/// as an untyped extras map.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct FinancialStatement {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub report_type: Option<String>,
    #[serde(default)]
    pub fiscal_year_end: Option<i64>,
    #[serde(default)]
    pub accession_number: Option<String>,
    #[serde(default)]
    pub filing_date: Option<NaiveDate>,
    #[serde(default)]
    pub form_type: Option<String>,
    #[serde(default)]
    pub period_ending_date: Option<NaiveDate>,
}

// -----------------------------------------------------------------------------
// 7. GET /beta/markets/fundamentals/statistics
// -----------------------------------------------------------------------------

/// One element of the response array for
/// `GET /beta/markets/fundamentals/statistics`.
pub type StatisticsResponse = FundamentalsEnvelope<StatisticsResult>;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct StatisticsResult {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tables: Option<StatisticsTables>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct StatisticsTables {
    #[serde(default)]
    pub price_statistics: Option<OneOrMany<PriceStatistics>>,
    #[serde(default)]
    pub trailing_returns: Option<OneOrMany<TrailingReturns>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct PriceStatistics {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub as_of_date: Option<NaiveDate>,
    #[serde(default)]
    pub moving_average_price: Option<f64>,
    #[serde(default)]
    pub close_price: Option<f64>,
    #[serde(default)]
    pub high_price: Option<f64>,
    #[serde(default)]
    pub low_price: Option<f64>,
    #[serde(default)]
    pub percent_change: Option<f64>,
    #[serde(default)]
    pub price_change: Option<f64>,
    #[serde(default)]
    pub total_volume: Option<f64>,
    #[serde(default)]
    pub average_volume: Option<f64>,
    #[serde(default)]
    pub arithmetic_mean: Option<f64>,
    #[serde(default)]
    pub standard_deviation: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct TrailingReturns {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub as_of_date: Option<NaiveDate>,
    #[serde(default)]
    pub total_return: Option<f64>,
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    use crate::fundamentals::test_support::{
        CompanyResponseArrayWire, CorporateActionResponseArrayWire,
        CorporateCalendarResponseArrayWire, DividendResponseArrayWire, FinancialsResponseArrayWire,
        RatiosResponseArrayWire, StatisticsResponseArrayWire,
    };

    #[test]
    fn test_envelope_default_results_when_missing() {
        let json = r#"{"request":"AAPL","type":"Symbol"}"#;
        let parsed: FundamentalsEnvelope<CompanyResult> =
            serde_json::from_str(json).expect("parse");
        assert_eq!(parsed.request, "AAPL");
        assert_eq!(parsed.kind, "Symbol");
        assert!(parsed.results.is_empty());
    }

    #[test]
    fn test_envelope_with_empty_results_parses() {
        let json = r#"{"request":"AAPL","type":"Symbol","results":[]}"#;
        let parsed: FundamentalsEnvelope<CompanyResult> =
            serde_json::from_str(json).expect("parse");
        assert!(parsed.results.is_empty());
    }

    #[test]
    fn test_envelope_with_one_result_parses() {
        let json = r#"{
            "request":"AAPL",
            "type":"Symbol",
            "results":[{"type":"Company","id":"x","tables":{}}]
        }"#;
        let parsed: FundamentalsEnvelope<CompanyResult> =
            serde_json::from_str(json).expect("parse");
        assert_eq!(parsed.results.len(), 1);
        assert_eq!(parsed.results[0].kind, "Company");
    }

    #[test]
    fn test_company_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Company",
                "id":"0P000000GY",
                "tables":{
                  "company_profile":{
                    "company_id":"0P000000GY",
                    "total_employee_number":150000,
                    "headquarter":{
                      "address_line1":"One Apple Park Way",
                      "city":"Cupertino",
                      "country":"USA",
                      "province":"CA"
                    }
                  },
                  "asset_classification":{
                    "financial_health_grade":"A",
                    "morningstar_sector_code":311
                  },
                  "share_class":{
                    "currency_id":"USD",
                    "exchange_id":"NAS",
                    "symbol":"AAPL",
                    "is_primary_share":true
                  }
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<CompanyResponse> = serde_json::from_str(json).expect("parse");
        let first = &parsed[0];
        assert_eq!(first.request, "AAPL");
        let result = &first.results[0];
        let tables = result.tables.as_ref().expect("tables");
        let profile = tables.company_profile.as_ref().expect("profile");
        assert_eq!(profile.total_employee_number, Some(150000));
        let hq = profile.headquarter.as_ref().expect("hq");
        assert_eq!(hq.city.as_deref(), Some("Cupertino"));
        let classification = tables
            .asset_classification
            .as_ref()
            .expect("classification");
        assert_eq!(classification.financial_health_grade.as_deref(), Some("A"));
    }

    #[test]
    fn test_dividends_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Stock",
                "id":"0P000000GY",
                "tables":{
                  "cash_dividends":[{
                    "share_class_id":"0P000000GY",
                    "dividend_type":"CD",
                    "ex_date":"2024-02-09",
                    "cash_amount":0.24,
                    "currency_i_d":"USD",
                    "declaration_date":"2024-02-01",
                    "frequency":4,
                    "pay_date":"2024-02-15",
                    "record_date":"2024-02-12"
                  }]
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<DividendResponse> = serde_json::from_str(json).expect("parse");
        assert_eq!(parsed.len(), 1);
        let first = &parsed[0];
        assert_eq!(first.request, "AAPL");
        let result = &first.results[0];
        let tables = result.tables.as_ref().expect("tables");
        let cash = tables.cash_dividends.as_ref().expect("cash").clone();
        let cash = cash.into_vec();
        assert_eq!(cash.len(), 1);
        assert_eq!(cash[0].dividend_type.as_deref(), Some("CD"));
        assert_eq!(cash[0].cash_amount, Some(0.24));
    }

    #[test]
    fn test_corporate_calendars_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Company",
                "tables":{
                  "corporate_calendars":[{
                    "company_id":"0P000000GY",
                    "begin_date_time":"2024-04-30",
                    "end_date_time":"2024-04-30",
                    "event_type":14,
                    "event":"Earnings Release",
                    "event_status":"Confirmed",
                    "time_zone":"US/Eastern"
                  }]
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<CorporateCalendarResponse> = serde_json::from_str(json).expect("parse");
        let events = parsed[0].results[0]
            .tables
            .as_ref()
            .and_then(|t| t.corporate_calendars.clone())
            .expect("events")
            .into_vec();
        assert_eq!(events[0].event.as_deref(), Some("Earnings Release"));
    }

    #[test]
    fn test_corporate_actions_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Stock",
                "tables":{
                  "stock_splits":[{
                    "share_class_id":"0P000000GY",
                    "ex_date":"2020-08-31",
                    "adjustment_factor":4.0,
                    "split_from":1.0,
                    "split_to":4.0,
                    "split_type":"SS"
                  }]
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<CorporateActionResponse> = serde_json::from_str(json).expect("parse");
        let splits = parsed[0].results[0]
            .tables
            .as_ref()
            .and_then(|t| t.stock_splits.clone())
            .expect("splits")
            .into_vec();
        assert_eq!(splits[0].split_to, Some(4.0));
    }

    #[test]
    fn test_ratios_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Stock",
                "tables":{
                  "valuation_ratios":[{
                    "as_of_date":"2024-04-01",
                    "p_e_ratio":28.5,
                    "p_b_ratio":35.0,
                    "dividend_yield":0.005
                  }]
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<RatiosResponse> = serde_json::from_str(json).expect("parse");
        let vr = parsed[0].results[0]
            .tables
            .as_ref()
            .and_then(|t| t.valuation_ratios.clone())
            .expect("vr")
            .into_vec();
        assert_eq!(vr[0].p_e_ratio, Some(28.5));
    }

    #[test]
    fn test_financials_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Stock",
                "tables":{
                  "income_statement":[{
                    "period":"FY",
                    "report_type":"A",
                    "fiscal_year_end":9,
                    "filing_date":"2023-11-03",
                    "form_type":"10-K",
                    "period_ending_date":"2023-09-30"
                  }]
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<FinancialsResponse> = serde_json::from_str(json).expect("parse");
        let income = parsed[0].results[0]
            .tables
            .as_ref()
            .and_then(|t| t.income_statement.clone())
            .expect("income")
            .into_vec();
        assert_eq!(income[0].form_type.as_deref(), Some("10-K"));
    }

    #[test]
    fn test_statistics_parses_real_shape() {
        let json = r#"[
          {
            "request":"AAPL",
            "type":"Symbol",
            "results":[
              {
                "type":"Stock",
                "tables":{
                  "price_statistics":[{
                    "period":"52W",
                    "as_of_date":"2024-04-01",
                    "close_price":170.25,
                    "high_price":199.62,
                    "low_price":124.17
                  }]
                }
              }
            ]
          }
        ]"#;
        let parsed: Vec<StatisticsResponse> = serde_json::from_str(json).expect("parse");
        let stats = parsed[0].results[0]
            .tables
            .as_ref()
            .and_then(|t| t.price_statistics.clone())
            .expect("stats")
            .into_vec();
        assert_eq!(stats[0].close_price, Some(170.25));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_deserialize_company_response_array(wire in any::<CompanyResponseArrayWire>()) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<CompanyResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}, json: {}", result, json);
        }

        #[test]
        fn test_deserialize_corporate_calendar_response_array(
            wire in any::<CorporateCalendarResponseArrayWire>()
        ) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<CorporateCalendarResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}", result);
        }

        #[test]
        fn test_deserialize_dividend_response_array(wire in any::<DividendResponseArrayWire>()) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<DividendResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}", result);
        }

        #[test]
        fn test_deserialize_corporate_action_response_array(
            wire in any::<CorporateActionResponseArrayWire>()
        ) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<CorporateActionResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}", result);
        }

        #[test]
        fn test_deserialize_ratios_response_array(wire in any::<RatiosResponseArrayWire>()) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<RatiosResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}", result);
        }

        #[test]
        fn test_deserialize_financials_response_array(
            wire in any::<FinancialsResponseArrayWire>()
        ) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<FinancialsResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}", result);
        }

        #[test]
        fn test_deserialize_statistics_response_array(
            wire in any::<StatisticsResponseArrayWire>()
        ) {
            let json = serde_json::to_string(&wire).expect("serialize");
            let result: std::result::Result<Vec<StatisticsResponse>, serde_json::Error> =
                serde_json::from_str(&json);
            prop_assert!(result.is_ok(), "failed: {:?}", result);
        }
    }
}
