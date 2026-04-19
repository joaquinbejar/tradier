//! Tradier WebSocket market event response types.
//!
//! See <https://documentation.tradier.com/brokerage-api/reference/response/streaming>
//! for the upstream contract.
//!
//! The Tradier market stream emits one JSON object per line. The `type`
//! discriminator selects the variant: `quote`, `trade`, `summary`,
//! `timesale`, or `tradex`. Some numeric fields come back as JSON strings
//! (trade / timesale / tradex prices and sizes) — the types below mirror
//! the upstream shape exactly, and helper methods (e.g. [`Trade::price_f64`])
//! parse strings to numbers at the call site rather than silently papering
//! over the mismatch.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// A market event received from the Tradier WebSocket stream.
///
/// This enum is tagged by the upstream `type` field. Unknown variants
/// are rejected with a deserialization error — callers that want
/// forward-compatibility should deserialize into [`serde_json::Value`]
/// first and then into [`MarketEvent`].
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarketEvent {
    /// Best bid and ask update for a symbol.
    Quote(Quote),
    /// Individual trade print.
    Trade(Trade),
    /// Daily summary (open / high / low / prev close).
    Summary(Summary),
    /// Tick-by-tick time-and-sales entry.
    Timesale(Timesale),
    /// Extended-hours / off-exchange trade.
    Tradex(Tradex),
}

/// Best bid / ask for a symbol.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub bidsz: u64,
    pub bidexch: String,
    /// Bid timestamp as a millisecond-epoch string.
    pub biddate: String,
    pub ask: f64,
    pub asksz: u64,
    pub askexch: String,
    /// Ask timestamp as a millisecond-epoch string.
    pub askdate: String,
}

/// Individual trade print. Prices and sizes are strings on the wire.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Trade {
    pub symbol: String,
    pub exch: String,
    pub price: String,
    pub size: String,
    pub cvol: String,
    /// Trade timestamp as a millisecond-epoch string.
    pub date: String,
    pub last: String,
}

impl Trade {
    /// Parses [`Trade::price`] from the upstream string to `f64`.
    ///
    /// # Errors
    /// Returns [`Error::ParseFloat`] if the upstream value is not a
    /// well-formed decimal number.
    #[inline]
    pub fn price_f64(&self) -> Result<f64> {
        parse_f64(&self.price)
    }

    /// Parses [`Trade::size`] from the upstream string to `u64`.
    ///
    /// # Errors
    /// Returns [`Error::ParseInt`] if the upstream value is not a
    /// well-formed integer.
    #[inline]
    pub fn size_u64(&self) -> Result<u64> {
        parse_u64(&self.size)
    }
}

/// Daily summary for a symbol.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Summary {
    pub symbol: String,
    pub open: String,
    pub high: String,
    pub low: String,
    #[serde(rename = "prevClose")]
    pub prev_close: String,
}

/// Time-and-sales entry.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Timesale {
    pub symbol: String,
    pub exch: String,
    pub bid: String,
    pub ask: String,
    pub last: String,
    pub size: String,
    /// Trade timestamp as a millisecond-epoch string.
    pub date: String,
    pub seq: u64,
    pub flag: String,
    pub cancel: bool,
    pub correction: bool,
    pub session: TradeSession,
}

/// Extended trade event (off-exchange / extended-hours / multi-leg).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Tradex {
    pub symbol: String,
    pub exch: String,
    pub price: String,
    pub size: String,
    pub cvol: String,
    /// Trade timestamp as a millisecond-epoch string.
    pub date: String,
    pub last: String,
}

/// Trading session label attached to a time-and-sales entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TradeSession {
    Pre,
    Normal,
    Post,
}

impl MarketEvent {
    /// Returns the symbol the event is for.
    #[must_use]
    #[inline]
    pub fn symbol(&self) -> &str {
        match self {
            MarketEvent::Quote(q) => &q.symbol,
            MarketEvent::Trade(t) => &t.symbol,
            MarketEvent::Summary(s) => &s.symbol,
            MarketEvent::Timesale(ts) => &ts.symbol,
            MarketEvent::Tradex(tx) => &tx.symbol,
        }
    }

    /// Parses a single market event from a JSON line.
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

#[cold]
fn parse_f64(value: &str) -> Result<f64> {
    f64::from_str(value).map_err(|e| Error::ParseFloat(value.to_owned(), e.to_string()))
}

#[cold]
fn parse_u64(value: &str) -> Result<u64> {
    u64::from_str(value).map_err(|e| Error::ParseInt(value.to_owned(), e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_from_json_happy_path() {
        let line = r#"{"type":"quote","symbol":"C","bid":281.84,"bidsz":60,"bidexch":"M","biddate":"1557757189000","ask":281.85,"asksz":6,"askexch":"Z","askdate":"1557757190000"}"#;
        let event = MarketEvent::from_json(line).expect("parse");
        let MarketEvent::Quote(quote) = event else {
            panic!("expected quote variant");
        };
        assert_eq!(quote.symbol, "C");
        assert!((quote.bid - 281.84).abs() < f64::EPSILON);
        assert_eq!(quote.bidsz, 60);
        assert_eq!(quote.biddate, "1557757189000");
    }

    #[test]
    fn test_trade_from_json_happy_path() {
        let line = r#"{"type":"trade","symbol":"SPY","exch":"Q","price":"281.1200","size":"100","cvol":"34507070","date":"1557757204760","last":"281.1200"}"#;
        let event = MarketEvent::from_json(line).expect("parse");
        let MarketEvent::Trade(trade) = event else {
            panic!("expected trade variant");
        };
        assert_eq!(trade.symbol, "SPY");
        assert!((trade.price_f64().expect("price") - 281.12).abs() < 1e-6);
        assert_eq!(trade.size_u64().expect("size"), 100);
    }

    #[test]
    fn test_summary_from_json_happy_path() {
        let line = r#"{"type":"summary","symbol":"SPY","open":"284.01","high":"284.42","low":"280.51","prevClose":"287.59"}"#;
        let event = MarketEvent::from_json(line).expect("parse");
        let MarketEvent::Summary(summary) = event else {
            panic!("expected summary variant");
        };
        assert_eq!(summary.prev_close, "287.59");
    }

    #[test]
    fn test_timesale_from_json_happy_path() {
        let line = r#"{"type":"timesale","symbol":"SPY","exch":"Q","bid":"281.09","ask":"281.10","last":"281.10","size":"100","date":"1557757204760","seq":352342,"flag":"","cancel":false,"correction":false,"session":"normal"}"#;
        let event = MarketEvent::from_json(line).expect("parse");
        let MarketEvent::Timesale(ts) = event else {
            panic!("expected timesale variant");
        };
        assert_eq!(ts.seq, 352342);
        assert!(!ts.cancel);
        assert_eq!(ts.session, TradeSession::Normal);
    }

    #[test]
    fn test_tradex_from_json_happy_path() {
        let line = r#"{"type":"tradex","symbol":"SPY","exch":"Q","price":"281.10","size":"100","cvol":"34507070","date":"1557757204760","last":"281.10"}"#;
        let event = MarketEvent::from_json(line).expect("parse");
        let MarketEvent::Tradex(tx) = event else {
            panic!("expected tradex variant");
        };
        assert_eq!(tx.symbol, "SPY");
    }

    #[test]
    fn test_market_event_symbol_returns_variant_symbol() {
        let q = MarketEvent::Quote(Quote {
            symbol: "AAPL".into(),
            bid: 0.0,
            bidsz: 0,
            bidexch: String::new(),
            biddate: String::new(),
            ask: 0.0,
            asksz: 0,
            askexch: String::new(),
            askdate: String::new(),
        });
        assert_eq!(q.symbol(), "AAPL");
    }

    #[test]
    fn test_market_event_from_json_unknown_type_returns_decode_error() {
        let line = r#"{"type":"unknown","symbol":"SPY"}"#;
        let result = MarketEvent::from_json(line);
        assert!(matches!(result, Err(Error::StreamDecodeError(_, _))));
    }

    #[test]
    fn test_market_event_from_json_malformed_returns_decode_error() {
        let result = MarketEvent::from_json("not json");
        assert!(matches!(result, Err(Error::StreamDecodeError(_, _))));
    }

    #[test]
    fn test_trade_price_f64_malformed_returns_parse_float_error() {
        let trade = Trade {
            symbol: "SPY".into(),
            exch: "Q".into(),
            price: "not-a-number".into(),
            size: "100".into(),
            cvol: "0".into(),
            date: "0".into(),
            last: "0".into(),
        };
        assert!(matches!(trade.price_f64(), Err(Error::ParseFloat(_, _))));
    }

    #[test]
    fn test_trade_size_u64_malformed_returns_parse_int_error() {
        let trade = Trade {
            symbol: "SPY".into(),
            exch: "Q".into(),
            price: "0".into(),
            size: "not-an-int".into(),
            cvol: "0".into(),
            date: "0".into(),
            last: "0".into(),
        };
        assert!(matches!(trade.size_u64(), Err(Error::ParseInt(_, _))));
    }
}
