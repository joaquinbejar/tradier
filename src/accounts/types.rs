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
            return Err(crate::Error::AccountIdParseError(s.to_owned()));
        }
        let mut is_valid_ascii = false;
        for c in s.chars().map(|c| c as u8) {
            if (0x20u8..0x7fu8).contains(&c) {
                is_valid_ascii = true;
            }
        }
        if is_valid_ascii {
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

    use crate::{
        accounts::{
            test_support::{GetAccountBalancesResponseWire, GetAccountPositionsResponseWire},
            types::GetAccountBalancesResponse,
        },
        types::{AccountNumber, GetAccountPositionsResponse},
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

    }
}
