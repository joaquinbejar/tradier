use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::common::AccountType;

#[derive(Debug)]
pub struct AccountNumber(String);

impl FromStr for AccountNumber {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(crate::Error::AccountIdParseError(s.to_owned()))
        } else if s.chars().all(|c| (0x20u8..0x7fu8).contains(&(c as u8))) {
            Ok(Self(s.to_string()))
        } else {
            Err(crate::Error::AccountIdParseError(s.to_string()))
        }
    }
}

impl std::fmt::Display for AccountNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Page(i32);

impl Page {
    pub fn new(page_number: i32) -> Self {
        Self(page_number)
    }
}

impl std::default::Default for Page {
    fn default() -> Self {
        Self::new(1)
    }
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl From<i32> for Page {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Limit(u32);

impl Limit {
    pub fn new(limit: u32) -> Self {
        Self(limit)
    }
}

impl std::default::Default for Limit {
    fn default() -> Self {
        Self::new(25)
    }
}

impl std::fmt::Display for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl From<u32> for Limit {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum EventType {
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

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            EventType::Trade => "trade",
            EventType::Option => "option",
            EventType::Ach => "ach",
            EventType::Wire => "wire",
            EventType::Dividend => "dividend",
            EventType::Fee => "fee",
            EventType::Tax => "tax",
            EventType::Journal => "journal",
            EventType::Check => "check",
            EventType::Transfer => "transfer",
            EventType::Adjustment => "adjustment",
        };
        f.write_str(value)
    }
}

/// Direction for sorting account gain/loss results.
///
/// Currently specific to `get_account_gain_loss`. May be moved to `crate::common`
/// if other endpoints require sort order control.
#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        })
    }
}

/// Field to sort by when querying account gain/loss.
///
/// Currently specific to `get_account_gain_loss`. May be moved to `crate::common`
/// if other endpoints share these sort options.
#[derive(Clone, Debug, PartialEq)]
pub enum GainLossSortBy {
    CloseDate,
    OpenDate,
    Symbol,
    GainLoss,
}

impl std::fmt::Display for GainLossSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GainLossSortBy::CloseDate => "closedate",
            GainLossSortBy::OpenDate => "opendate",
            GainLossSortBy::Symbol => "symbol",
            GainLossSortBy::GainLoss => "gainloss",
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetAccountGainLossResponse {
    gainloss: AccountGainLoss,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AccountGainLoss {
    closed_position: Vec<ClosedPosition>,
    page: u32,
    total_pages: u32,
    total_positions: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ClosedPosition {
    close_date: DateTime<Utc>,
    cost: f64,
    gain_loss: f64,
    gain_loss_percent: f64,
    open_date: DateTime<Utc>,
    proceeds: f64,
    quantity: f64,
    symbol: String,
    term: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetAccountBalancesResponse {
    balances: AccountBalances,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AccountBalances {
    option_short_value: f64,
    total_equity: f64,
    account_number: String,
    account_type: AccountType,
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
    margin: Margin,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Margin {
    fed_call: f64,
    maintenance_call: f64,
    option_buying_power: f64,
    stock_buying_power: f64,
    stock_short_value: f64,
    sweep: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Position {
    cost_basis: f64,
    date_acquired: DateTime<Utc>,
    id: u32,
    quantity: f64,
    symbol: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetAccountPositionsResponse {
    positions: Vec<Position>,
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::{
        AccountNumber, EventType, GetAccountBalancesResponse, GetAccountGainLossResponse,
        GetAccountPositionsResponse, Limit, Page,
    };
    use crate::{
        accounts::test_support::{
            GetAccountBalancesResponseWire, GetAccountGainLossResponseWire,
            GetAccountPositionsResponseWire,
        },
        Result,
    };

    #[test]
    fn test_account_number_cannot_be_empty_or_blank() {
        let account_number = "".parse::<AccountNumber>();
        assert!(account_number.is_err());
        let account_number = "     ".parse::<AccountNumber>();
        assert!(account_number.is_err());
    }

    proptest! {
        #[test]
        fn test_deserialize_account_balances_response_from_json(response in any::<GetAccountBalancesResponseWire>()) {

            let response = serde_json::to_string_pretty(&response)
                .expect("test fixture to serialize");
            let result: std::result::Result<GetAccountBalancesResponse, serde_json::Error> = serde_json::from_str(&response);
            assert!(result.is_ok());
        }

        #[test]
        fn test_account_number_from_printable_ascii_string(ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..1000)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })) {

            let account_number: Result<AccountNumber> = ascii_string.parse();
            assert!(account_number.is_ok());
        }

        #[test]
        fn test_account_number_from_non_printable_ascii_string(ascii_string in prop::collection::vec(0x00u8..0x20u8, 1..1000)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })) {
            let account_number: Result<AccountNumber> = ascii_string.parse();
            assert!(account_number.is_err());
        }

        #[test]
        fn test_deserialize_positions_from_json(response in any::<GetAccountPositionsResponseWire>()) {
            let response = serde_json::to_string_pretty(&response)
                .expect("test fixture to serialize");
            let result: std::result::Result<GetAccountPositionsResponse, serde_json::Error> = serde_json::from_str(&response);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_gain_loss_response_from_json(
            response in any::<GetAccountGainLossResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response)
                .expect("test fixture to serialize");
            let result: std::result::Result<GetAccountGainLossResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_account_number_mixed_invalid() {
        // Mixed string containing a non-printable character should fail parsing.
        let input = "ABC\u{001}DEF";
        let result = input.parse::<AccountNumber>();
        assert!(result.is_err());
    }

    #[test]
    fn test_account_number_all_valid() {
        let input = "ValidAccount123";
        let result = input.parse::<AccountNumber>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_account_number_display_preserves_input() {
        let input = "Account 123";
        let account_number: AccountNumber = input.parse().expect("should parse");
        assert_eq!(account_number.to_string(), input);
    }

    #[test]
    fn test_account_number_rejects_high_ascii() {
        let input = "ID\u{80}";
        let account_number = input.parse::<AccountNumber>();
        assert!(account_number.is_err());
    }

    #[test]
    fn test_page_display() {
        let page = Page::new(5);
        assert_eq!(page.to_string(), "5");
    }

    #[test]
    fn test_page_default() {
        assert_eq!(Page::default(), Page::new(1));
    }

    #[test]
    fn test_page_from() {
        assert_eq!(Page::from(3), Page::new(3));
    }

    #[test]
    fn test_limit_display() {
        let limit = Limit::new(100);
        assert_eq!(limit.to_string(), "100");
    }

    #[test]
    fn test_limit_default() {
        assert_eq!(Limit::default(), Limit::new(25));
    }

    #[test]
    fn test_limit_from() {
        assert_eq!(Limit::from(10), Limit::new(10));
    }

    #[test]
    fn test_event_type_display_values() {
        let cases = vec![
            (EventType::Trade, "trade"),
            (EventType::Option, "option"),
            (EventType::Ach, "ach"),
            (EventType::Wire, "wire"),
            (EventType::Dividend, "dividend"),
            (EventType::Fee, "fee"),
            (EventType::Tax, "tax"),
            (EventType::Journal, "journal"),
            (EventType::Check, "check"),
            (EventType::Transfer, "transfer"),
            (EventType::Adjustment, "adjustment"),
        ];

        for (event, expected) in cases {
            assert_eq!(event.to_string(), expected);
        }
    }
}
