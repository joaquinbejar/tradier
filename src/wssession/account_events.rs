//! Tradier WebSocket account event response types.
//!
//! See <https://documentation.tradier.com/brokerage-api/streaming/wss-account-websocket>
//! for the upstream contract.
//!
//! The account stream emits one JSON object per line. The top-level
//! `event` field discriminates the variant: `order`, `fill`, `position`,
//! `balance`, `trade`, `drop`.
//!
//! Numeric fields are taken at the upstream type — Tradier sends
//! quantities as `f64` and timestamps as millisecond-epoch strings, so
//! we mirror that shape exactly. Callers that need decimal precision
//! convert at the boundary rather than have the library silently coerce.
//!
//! Variants are `#[non_exhaustive]` so new event kinds can be added in
//! a minor release without a breaking change.

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// An account event received from the Tradier WebSocket stream.
///
/// Tagged on the upstream `event` field. Unknown variants are rejected
/// with a deserialization error — callers that want
/// forward-compatibility should deserialize into [`serde_json::Value`]
/// first and then into [`AccountEvent`].
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "lowercase")]
#[non_exhaustive]
pub enum AccountEvent {
    /// Order lifecycle update (new, partially filled, filled,
    /// cancelled, rejected, ...).
    Order(AccountOrderEvent),
    /// Individual fill (execution) event attached to an order.
    Fill(AccountFillEvent),
    /// Position update (quantity / cost basis changed).
    Position(AccountPositionEvent),
    /// Balance update (cash / buying power / equity).
    Balance(AccountBalanceEvent),
    /// Aggregated trade event.
    Trade(AccountTradeEvent),
    /// Session dropped / account unsubscribed.
    Drop(AccountDropEvent),
}

/// Order lifecycle event.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountOrderEvent {
    /// Tradier order id.
    pub id: u64,
    /// Order status (`open`, `partially_filled`, `filled`, `canceled`,
    /// `rejected`, ...). Left as `String` so new upstream values do
    /// not break deserialization.
    pub status: String,
    /// Account number the order belongs to.
    #[serde(
        rename = "account_number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_number: Option<String>,
    /// Symbol the order is against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Side (`buy`, `sell`, `buy_to_cover`, `sell_short`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    /// Requested quantity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Remaining quantity (unfilled).
    #[serde(
        rename = "remaining_quantity",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub remaining_quantity: Option<f64>,
    /// Executed / filled quantity.
    #[serde(
        rename = "executed_quantity",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub executed_quantity: Option<f64>,
    /// Average fill price across all fills.
    #[serde(
        rename = "avg_fill_price",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub avg_fill_price: Option<f64>,
    /// Most recent fill price.
    #[serde(
        rename = "last_fill_price",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_fill_price: Option<f64>,
    /// Most recent fill quantity.
    #[serde(
        rename = "last_fill_quantity",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_fill_quantity: Option<f64>,
    /// Event timestamp (millisecond-epoch string on the wire).
    #[serde(
        rename = "transaction_date",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transaction_date: Option<String>,
}

/// Individual execution / fill attached to an order.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountFillEvent {
    /// Parent order id.
    #[serde(rename = "order_id", default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,
    /// Account number.
    #[serde(
        rename = "account_number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_number: Option<String>,
    /// Symbol that filled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Side of the fill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    /// Filled quantity on this print.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Fill price on this print.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Execution timestamp.
    #[serde(
        rename = "transaction_date",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transaction_date: Option<String>,
}

/// Position update.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountPositionEvent {
    /// Account number.
    #[serde(
        rename = "account_number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_number: Option<String>,
    /// Position symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// New signed quantity (positive = long, negative = short).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Total cost basis of the position.
    #[serde(
        rename = "cost_basis",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cost_basis: Option<f64>,
    /// Update timestamp.
    #[serde(
        rename = "date_acquired",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub date_acquired: Option<String>,
}

/// Balance update.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountBalanceEvent {
    /// Account number.
    #[serde(
        rename = "account_number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_number: Option<String>,
    /// Total account value (cash + long-market-value - short-market-value).
    #[serde(
        rename = "total_equity",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_equity: Option<f64>,
    /// Available cash.
    #[serde(
        rename = "total_cash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_cash: Option<f64>,
    /// Buying power.
    #[serde(
        rename = "buying_power",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub buying_power: Option<f64>,
    /// Market value of long positions.
    #[serde(
        rename = "long_market_value",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub long_market_value: Option<f64>,
    /// Market value of short positions.
    #[serde(
        rename = "short_market_value",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub short_market_value: Option<f64>,
}

/// Aggregated trade event (per-symbol, per-side rollup).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountTradeEvent {
    /// Account number.
    #[serde(
        rename = "account_number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_number: Option<String>,
    /// Symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Side (`buy`, `sell`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    /// Aggregated quantity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Aggregated volume-weighted average price.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Timestamp.
    #[serde(
        rename = "transaction_date",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transaction_date: Option<String>,
}

/// Session drop notification.
///
/// Tradier emits this when the account stream is terminated — e.g.
/// session expiry, administrative unsubscribe. Callers should treat
/// this as a reason to tear down the session and re-bootstrap.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountDropEvent {
    /// Account number affected by the drop.
    #[serde(
        rename = "account_number",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_number: Option<String>,
    /// Free-form reason string, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl AccountEvent {
    /// Returns the account number attached to the event, if any.
    #[must_use]
    #[inline]
    pub fn account_number(&self) -> Option<&str> {
        match self {
            AccountEvent::Order(o) => o.account_number.as_deref(),
            AccountEvent::Fill(f) => f.account_number.as_deref(),
            AccountEvent::Position(p) => p.account_number.as_deref(),
            AccountEvent::Balance(b) => b.account_number.as_deref(),
            AccountEvent::Trade(t) => t.account_number.as_deref(),
            AccountEvent::Drop(d) => d.account_number.as_deref(),
        }
    }

    /// Parses a single account event from a JSON line.
    ///
    /// # Errors
    /// Returns [`Error::StreamDecodeError`] with the offending payload
    /// and the underlying `serde_json` error when the JSON does not
    /// match any known variant.
    pub fn from_json(line: &str) -> Result<Self> {
        serde_json::from_str::<Self>(line)
            .map_err(|e| Error::StreamDecodeError(line.to_owned(), e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_event_order_happy_path() {
        let line = r#"{"event":"order","id":123,"status":"filled","account_number":"VA1234","symbol":"SPY","side":"buy","quantity":100.0,"remaining_quantity":0.0,"executed_quantity":100.0,"avg_fill_price":281.12,"transaction_date":"1557757204760"}"#;
        let event = AccountEvent::from_json(line).expect("parse");
        let AccountEvent::Order(order) = event else {
            panic!("expected order variant");
        };
        assert_eq!(order.id, 123);
        assert_eq!(order.status, "filled");
        assert_eq!(order.account_number.as_deref(), Some("VA1234"));
        assert_eq!(order.executed_quantity, Some(100.0));
    }

    #[test]
    fn test_account_event_fill_happy_path() {
        let line = r#"{"event":"fill","order_id":123,"account_number":"VA1234","symbol":"SPY","side":"buy","quantity":50.0,"price":281.12,"transaction_date":"1557757204760"}"#;
        let event = AccountEvent::from_json(line).expect("parse");
        let AccountEvent::Fill(fill) = event else {
            panic!("expected fill variant");
        };
        assert_eq!(fill.order_id, Some(123));
        assert_eq!(fill.quantity, Some(50.0));
    }

    #[test]
    fn test_account_event_position_happy_path() {
        let line = r#"{"event":"position","account_number":"VA1234","symbol":"SPY","quantity":100.0,"cost_basis":28112.0}"#;
        let event = AccountEvent::from_json(line).expect("parse");
        let AccountEvent::Position(p) = event else {
            panic!("expected position variant");
        };
        assert_eq!(p.cost_basis, Some(28112.0));
    }

    #[test]
    fn test_account_event_balance_happy_path() {
        let line = r#"{"event":"balance","account_number":"VA1234","total_equity":100000.0,"total_cash":50000.0,"buying_power":75000.0}"#;
        let event = AccountEvent::from_json(line).expect("parse");
        let AccountEvent::Balance(b) = event else {
            panic!("expected balance variant");
        };
        assert_eq!(b.total_equity, Some(100000.0));
    }

    #[test]
    fn test_account_event_trade_happy_path() {
        let line = r#"{"event":"trade","account_number":"VA1234","symbol":"SPY","side":"buy","quantity":100.0,"price":281.12}"#;
        let event = AccountEvent::from_json(line).expect("parse");
        let AccountEvent::Trade(t) = event else {
            panic!("expected trade variant");
        };
        assert_eq!(t.price, Some(281.12));
    }

    #[test]
    fn test_account_event_drop_happy_path() {
        let line = r#"{"event":"drop","account_number":"VA1234","reason":"session expired"}"#;
        let event = AccountEvent::from_json(line).expect("parse");
        let AccountEvent::Drop(d) = event else {
            panic!("expected drop variant");
        };
        assert_eq!(d.reason.as_deref(), Some("session expired"));
    }

    #[test]
    fn test_account_event_from_json_unknown_event_returns_decode_error() {
        let line = r#"{"event":"unknown","id":1}"#;
        let result = AccountEvent::from_json(line);
        assert!(matches!(result, Err(Error::StreamDecodeError(_, _))));
    }

    #[test]
    fn test_account_event_from_json_malformed_returns_decode_error() {
        let result = AccountEvent::from_json("not json");
        assert!(matches!(result, Err(Error::StreamDecodeError(_, _))));
    }

    #[test]
    fn test_account_event_account_number_returns_value_when_present() {
        let order = AccountOrderEvent {
            id: 1,
            status: "open".into(),
            account_number: Some("VA9".into()),
            symbol: None,
            side: None,
            quantity: None,
            remaining_quantity: None,
            executed_quantity: None,
            avg_fill_price: None,
            last_fill_price: None,
            last_fill_quantity: None,
            transaction_date: None,
        };
        let event = AccountEvent::Order(order);
        assert_eq!(event.account_number(), Some("VA9"));
    }
}
