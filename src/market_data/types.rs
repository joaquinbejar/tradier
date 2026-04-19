//! Request and response types for the Tradier Market Data REST endpoints.
//!
//! See upstream documentation at
//! <https://documentation.tradier.com/brokerage-api/markets/>.

use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use crate::utils::OneOrMany;

// -----------------------------------------------------------------------------
// Shared newtypes & enums
// -----------------------------------------------------------------------------

/// A ticker symbol (equity, option, index, etc.).
///
/// Tradier symbols are ASCII printable strings; this wrapper validates the
/// input and rejects empty / blank values.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol(String);

impl Symbol {
    /// Returns the inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Symbol {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(crate::Error::MarketDataParseError(format!(
                "invalid symbol: '{s}' must not be empty or blank"
            )))
        } else if s.chars().all(|c| (0x20u8..0x7fu8).contains(&(c as u8))) {
            Ok(Self(s.to_string()))
        } else {
            Err(crate::Error::MarketDataParseError(format!(
                "invalid symbol: '{s}' must be printable ASCII"
            )))
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Comma-separated collection of [`Symbol`]s used by quote endpoints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Symbols(Vec<Symbol>);

impl Symbols {
    /// Creates a new [`Symbols`] from an iterator of [`Symbol`].
    #[must_use]
    pub fn new(symbols: impl IntoIterator<Item = Symbol>) -> Self {
        Self(symbols.into_iter().collect())
    }

    /// Returns the inner slice of [`Symbol`] values.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Symbol] {
        &self.0
    }

    /// Returns `true` if no symbols have been added.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for Symbols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for s in &self.0 {
            if !first {
                f.write_str(",")?;
            }
            f.write_str(s.as_str())?;
            first = false;
        }
        Ok(())
    }
}

/// Controls whether to include Greeks on option quote responses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Greeks(bool);

impl Greeks {
    /// Constructs a new [`Greeks`] flag.
    #[inline]
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Greeks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for Greeks {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Historical data bar granularity.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HistoryInterval {
    Daily,
    Weekly,
    Monthly,
}

impl std::fmt::Display for HistoryInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HistoryInterval::Daily => "daily",
            HistoryInterval::Weekly => "weekly",
            HistoryInterval::Monthly => "monthly",
        })
    }
}

/// Time-and-sales data bar granularity.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeSalesInterval {
    Tick,
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
}

impl std::fmt::Display for TimeSalesInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TimeSalesInterval::Tick => "tick",
            TimeSalesInterval::OneMinute => "1min",
            TimeSalesInterval::FiveMinutes => "5min",
            TimeSalesInterval::FifteenMinutes => "15min",
        })
    }
}

/// Session filter for history / timesales endpoints.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SessionFilter {
    All,
    Open,
}

impl std::fmt::Display for SessionFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SessionFilter::All => "all",
            SessionFilter::Open => "open",
        })
    }
}

// -----------------------------------------------------------------------------
// 1 & 2. GET / POST /v1/markets/quotes
// -----------------------------------------------------------------------------

/// Response to `GET|POST /v1/markets/quotes`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetQuotesResponse {
    pub quotes: QuotesPayload,
}

/// Body of the `quotes` response. Tradier returns `quote` as either a single
/// object, an array, or omits it when all symbols were unmatched.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct QuotesPayload {
    #[serde(default)]
    pub quote: Option<OneOrMany<Quote>>,
    #[serde(default)]
    pub unmatched_symbols: Option<UnmatchedSymbols>,
}

/// Wrapper for Tradier's `unmatched_symbols` block which may be a single symbol
/// string or an array.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UnmatchedSymbols {
    pub symbol: OneOrMany<String>,
}

/// A single quote row returned by the quotes endpoint.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Quote {
    pub symbol: String,
    pub description: String,
    pub exch: String,
    #[serde(rename = "type")]
    pub quote_type: String,
    #[serde(default)]
    pub last: Option<f64>,
    #[serde(default)]
    pub change: Option<f64>,
    #[serde(default)]
    pub volume: Option<u64>,
    #[serde(default)]
    pub open: Option<f64>,
    #[serde(default)]
    pub high: Option<f64>,
    #[serde(default)]
    pub low: Option<f64>,
    #[serde(default)]
    pub close: Option<f64>,
    #[serde(default)]
    pub bid: Option<f64>,
    #[serde(default)]
    pub ask: Option<f64>,
    #[serde(default)]
    pub change_percentage: Option<f64>,
    #[serde(default)]
    pub average_volume: Option<u64>,
    #[serde(default)]
    pub last_volume: Option<u64>,
    #[serde(default)]
    pub trade_date: Option<i64>,
    #[serde(default)]
    pub prevclose: Option<f64>,
    #[serde(default)]
    pub week_52_high: Option<f64>,
    #[serde(default)]
    pub week_52_low: Option<f64>,
    #[serde(default)]
    pub bidsize: Option<u64>,
    #[serde(default)]
    pub bidexch: Option<String>,
    #[serde(default)]
    pub bid_date: Option<i64>,
    #[serde(default)]
    pub asksize: Option<u64>,
    #[serde(default)]
    pub askexch: Option<String>,
    #[serde(default)]
    pub ask_date: Option<i64>,
    #[serde(default)]
    pub root_symbols: Option<String>,
    // Option-specific fields
    #[serde(default)]
    pub underlying: Option<String>,
    #[serde(default)]
    pub strike: Option<f64>,
    #[serde(default)]
    pub open_interest: Option<u64>,
    #[serde(default)]
    pub contract_size: Option<u64>,
    #[serde(default)]
    pub expiration_date: Option<String>,
    #[serde(default)]
    pub expiration_type: Option<String>,
    #[serde(default)]
    pub option_type: Option<String>,
    #[serde(default)]
    pub root_symbol: Option<String>,
    #[serde(default)]
    pub greeks: Option<GreeksData>,
}

/// Greeks block attached to an option quote when requested.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GreeksData {
    #[serde(default)]
    pub delta: Option<f64>,
    #[serde(default)]
    pub gamma: Option<f64>,
    #[serde(default)]
    pub theta: Option<f64>,
    #[serde(default)]
    pub vega: Option<f64>,
    #[serde(default)]
    pub rho: Option<f64>,
    #[serde(default)]
    pub phi: Option<f64>,
    #[serde(default)]
    pub bid_iv: Option<f64>,
    #[serde(default)]
    pub mid_iv: Option<f64>,
    #[serde(default)]
    pub ask_iv: Option<f64>,
    #[serde(default)]
    pub smv_vol: Option<f64>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

// -----------------------------------------------------------------------------
// 3. GET /v1/markets/options/chains
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/options/chains`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetOptionChainsResponse {
    pub options: Option<OptionChainsPayload>,
}

/// Body of the options chains response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionChainsPayload {
    pub option: OneOrMany<Quote>,
}

// -----------------------------------------------------------------------------
// 4. GET /v1/markets/options/strikes
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/options/strikes`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetOptionStrikesResponse {
    pub strikes: OptionStrikesPayload,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionStrikesPayload {
    pub strike: OneOrMany<f64>,
}

// -----------------------------------------------------------------------------
// 5. GET /v1/markets/options/expirations
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/options/expirations`.
///
/// The response shape changes depending on the `strikes` query flag:
/// - without strikes: `{ "expirations": { "date": ["..."] } }`
/// - with strikes:    `{ "expirations": { "expiration": [{ "date", "strikes" }] } }`
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetOptionExpirationsResponse {
    pub expirations: OptionExpirationsPayload,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OptionExpirationsPayload {
    #[serde(default)]
    pub date: Option<OneOrMany<String>>,
    #[serde(default)]
    pub expiration: Option<OneOrMany<ExpirationEntry>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ExpirationEntry {
    pub date: String,
    #[serde(default)]
    pub contract_size: Option<u64>,
    #[serde(default)]
    pub expiration_type: Option<String>,
    #[serde(default)]
    pub strikes: Option<ExpirationStrikes>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ExpirationStrikes {
    pub strike: OneOrMany<f64>,
}

/// Controls whether expirations endpoint returns all roots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IncludeAllRoots(bool);

impl IncludeAllRoots {
    #[inline]
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for IncludeAllRoots {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for IncludeAllRoots {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Controls whether expirations endpoint also returns strikes per expiration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IncludeStrikes(bool);

impl IncludeStrikes {
    #[inline]
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for IncludeStrikes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for IncludeStrikes {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

// -----------------------------------------------------------------------------
// 6. GET /v1/markets/options/lookup
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/options/lookup`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LookupOptionSymbolsResponse {
    pub symbols: OneOrMany<LookupOptionSymbolsRoot>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LookupOptionSymbolsRoot {
    #[serde(rename = "rootSymbol")]
    pub root_symbol: String,
    pub options: OneOrMany<String>,
}

// -----------------------------------------------------------------------------
// 7. GET /v1/markets/history
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/history`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetHistoricalQuotesResponse {
    pub history: Option<HistoryPayload>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HistoryPayload {
    pub day: OneOrMany<HistoryDay>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HistoryDay {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

// -----------------------------------------------------------------------------
// 8. GET /v1/markets/timesales
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/timesales`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetTimeAndSalesResponse {
    pub series: Option<TimeSalesPayload>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TimeSalesPayload {
    pub data: OneOrMany<TimeSalesEntry>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TimeSalesEntry {
    pub time: String,
    pub timestamp: i64,
    pub price: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    #[serde(default)]
    pub vwap: Option<f64>,
}

// -----------------------------------------------------------------------------
// 9. GET /v1/markets/etb
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/etb`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetEtbSecuritiesResponse {
    pub securities: Option<EtbSecurities>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct EtbSecurities {
    pub security: OneOrMany<EtbSecurity>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct EtbSecurity {
    pub symbol: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub security_type: String,
    pub description: String,
}

// -----------------------------------------------------------------------------
// 10. GET /v1/markets/clock
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/clock`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetClockResponse {
    pub clock: MarketClock,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MarketClock {
    pub date: String,
    pub description: String,
    pub state: String,
    pub timestamp: i64,
    pub next_change: String,
    pub next_state: String,
}

/// Controls whether the clock endpoint returns the delayed state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DelayedFlag(bool);

impl DelayedFlag {
    #[inline]
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for DelayedFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for DelayedFlag {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

// -----------------------------------------------------------------------------
// 11. GET /v1/markets/calendar
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/calendar`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GetCalendarResponse {
    pub calendar: MarketCalendar,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MarketCalendar {
    pub month: u32,
    pub year: u32,
    pub days: CalendarDays,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalendarDays {
    pub day: OneOrMany<CalendarDay>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalendarDay {
    pub date: NaiveDate,
    pub status: String,
    pub description: String,
    #[serde(default)]
    pub premarket: Option<CalendarWindow>,
    #[serde(default)]
    pub open: Option<CalendarWindow>,
    #[serde(default)]
    pub postmarket: Option<CalendarWindow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalendarWindow {
    pub start: String,
    pub end: String,
}

/// Calendar month (1-12) newtype.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarMonth(u32);

impl CalendarMonth {
    /// Creates a new [`CalendarMonth`], returning an error if the value is not in `1..=12`.
    ///
    /// # Errors
    /// Returns [`crate::Error::MarketDataParseError`] when the value is outside `1..=12`.
    pub fn new(value: u32) -> crate::Result<Self> {
        if (1..=12).contains(&value) {
            Ok(Self(value))
        } else {
            Err(crate::Error::MarketDataParseError(format!(
                "invalid calendar month: {value}, must be in 1..=12"
            )))
        }
    }
}

impl std::fmt::Display for CalendarMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Calendar year (e.g. 2024).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarYear(u32);

impl CalendarYear {
    #[inline]
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for CalendarYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -----------------------------------------------------------------------------
// 12. GET /v1/markets/search
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/search`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SearchCompaniesResponse {
    pub securities: Option<SearchSecurities>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SearchSecurities {
    pub security: OneOrMany<SearchSecurity>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SearchSecurity {
    pub symbol: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub security_type: String,
    pub description: String,
}

/// Controls whether the search endpoint restricts to indices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndexesFlag(bool);

impl IndexesFlag {
    #[inline]
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for IndexesFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for IndexesFlag {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

// -----------------------------------------------------------------------------
// 13. GET /v1/markets/lookup
// -----------------------------------------------------------------------------

/// Response to `GET /v1/markets/lookup`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LookupSymbolResponse {
    pub securities: Option<LookupSecurities>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LookupSecurities {
    pub security: OneOrMany<LookupSecurity>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LookupSecurity {
    pub symbol: String,
    pub exchange: String,
    #[serde(rename = "type")]
    pub security_type: String,
    pub description: String,
}

/// Comma-separated exchange filter for the lookup endpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exchanges(Vec<String>);

impl Exchanges {
    #[must_use]
    pub fn new(values: impl IntoIterator<Item = String>) -> Self {
        Self(values.into_iter().collect())
    }

    /// Returns `true` if no exchanges were provided.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for Exchanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.join(","))
    }
}

/// Comma-separated security type filter for the lookup endpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SecurityTypes(Vec<String>);

impl SecurityTypes {
    #[must_use]
    pub fn new(values: impl IntoIterator<Item = String>) -> Self {
        Self(values.into_iter().collect())
    }

    /// Returns `true` if no types were provided.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for SecurityTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.join(","))
    }
}

// -----------------------------------------------------------------------------
// Helpers to format dates for the wire
// -----------------------------------------------------------------------------

/// Format a [`NaiveDate`] as `YYYY-MM-DD` for Tradier query strings.
#[inline]
#[must_use]
pub fn format_naive_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Format a [`DateTime<Utc>`] as `YYYY-MM-DD HH:MM` for timesales queries.
#[inline]
#[must_use]
pub fn format_timesales_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

    use crate::market_data::test_support::{
        GetCalendarResponseWire, GetClockResponseWire, GetEtbSecuritiesResponseWire,
        GetHistoricalQuotesResponseWire, GetOptionChainsResponseWire,
        GetOptionExpirationsResponseWire, GetOptionStrikesResponseWire, GetQuotesResponseWire,
        GetTimeAndSalesResponseWire, LookupOptionSymbolsResponseWire, LookupSymbolResponseWire,
        SearchCompaniesResponseWire,
    };

    #[test]
    fn test_symbol_empty_should_error() {
        let result: Result<Symbol, _> = "".parse();
        assert!(result.is_err());
        let result: Result<Symbol, _> = "   ".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_valid_should_succeed() {
        let result: Result<Symbol, _> = "AAPL".parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "AAPL");
    }

    #[test]
    fn test_symbol_rejects_non_ascii() {
        let result: Result<Symbol, _> = "BAD\u{80}".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_symbols_display_comma_separated() {
        let syms = Symbols::new(vec![
            "AAPL".parse().unwrap(),
            "MSFT".parse().unwrap(),
            "GOOG".parse().unwrap(),
        ]);
        assert_eq!(syms.to_string(), "AAPL,MSFT,GOOG");
    }

    #[test]
    fn test_symbols_empty_display() {
        let syms = Symbols::default();
        assert_eq!(syms.to_string(), "");
        assert!(syms.is_empty());
    }

    #[test]
    fn test_history_interval_display_values() {
        assert_eq!(HistoryInterval::Daily.to_string(), "daily");
        assert_eq!(HistoryInterval::Weekly.to_string(), "weekly");
        assert_eq!(HistoryInterval::Monthly.to_string(), "monthly");
    }

    #[test]
    fn test_timesales_interval_display_values() {
        assert_eq!(TimeSalesInterval::Tick.to_string(), "tick");
        assert_eq!(TimeSalesInterval::OneMinute.to_string(), "1min");
        assert_eq!(TimeSalesInterval::FiveMinutes.to_string(), "5min");
        assert_eq!(TimeSalesInterval::FifteenMinutes.to_string(), "15min");
    }

    #[test]
    fn test_session_filter_display() {
        assert_eq!(SessionFilter::All.to_string(), "all");
        assert_eq!(SessionFilter::Open.to_string(), "open");
    }

    #[test]
    fn test_calendar_month_valid() {
        assert!(CalendarMonth::new(1).is_ok());
        assert!(CalendarMonth::new(12).is_ok());
        assert_eq!(CalendarMonth::new(6).unwrap().to_string(), "6");
    }

    #[test]
    fn test_calendar_month_invalid() {
        assert!(CalendarMonth::new(0).is_err());
        assert!(CalendarMonth::new(13).is_err());
        assert!(CalendarMonth::new(100).is_err());
    }

    #[test]
    fn test_calendar_year_display() {
        assert_eq!(CalendarYear::new(2024).to_string(), "2024");
    }

    #[test]
    fn test_greeks_display() {
        assert_eq!(Greeks::new(true).to_string(), "true");
        assert_eq!(Greeks::new(false).to_string(), "false");
        assert_eq!(Greeks::from(true).to_string(), "true");
    }

    #[test]
    fn test_delayed_flag_display() {
        assert_eq!(DelayedFlag::new(true).to_string(), "true");
        assert_eq!(DelayedFlag::new(false).to_string(), "false");
        assert_eq!(DelayedFlag::from(true).to_string(), "true");
    }

    #[test]
    fn test_include_all_roots_display() {
        assert_eq!(IncludeAllRoots::new(true).to_string(), "true");
        assert_eq!(IncludeAllRoots::from(false).to_string(), "false");
    }

    #[test]
    fn test_include_strikes_display() {
        assert_eq!(IncludeStrikes::new(true).to_string(), "true");
        assert_eq!(IncludeStrikes::from(false).to_string(), "false");
    }

    #[test]
    fn test_indexes_flag_display() {
        assert_eq!(IndexesFlag::new(true).to_string(), "true");
        assert_eq!(IndexesFlag::from(false).to_string(), "false");
    }

    #[test]
    fn test_exchanges_display() {
        let ex = Exchanges::new(vec!["N".to_string(), "Q".to_string(), "A".to_string()]);
        assert_eq!(ex.to_string(), "N,Q,A");
        assert!(!ex.is_empty());
    }

    #[test]
    fn test_exchanges_empty_display() {
        let ex = Exchanges::default();
        assert_eq!(ex.to_string(), "");
        assert!(ex.is_empty());
    }

    #[test]
    fn test_security_types_display() {
        let t = SecurityTypes::new(vec!["stock".to_string(), "etf".to_string()]);
        assert_eq!(t.to_string(), "stock,etf");
        assert!(!t.is_empty());
    }

    #[test]
    fn test_security_types_empty_display() {
        let t = SecurityTypes::default();
        assert_eq!(t.to_string(), "");
        assert!(t.is_empty());
    }

    #[test]
    fn test_format_naive_date() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).expect("valid date");
        assert_eq!(format_naive_date(&d), "2024-01-15");
    }

    #[test]
    fn test_format_timesales_datetime() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, 0).unwrap();
        assert_eq!(format_timesales_datetime(&dt), "2024-01-15 09:30");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn test_deserialize_quotes_response_from_json(response in any::<GetQuotesResponseWire>()) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetQuotesResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok(), "failed to deserialize: {:?}, json: {}", result, json);
        }

        #[test]
        fn test_deserialize_option_chains_response_from_json(
            response in any::<GetOptionChainsResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetOptionChainsResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok(), "failed: {:?}", result);
        }

        #[test]
        fn test_deserialize_option_strikes_response_from_json(
            response in any::<GetOptionStrikesResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetOptionStrikesResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_option_expirations_response_from_json(
            response in any::<GetOptionExpirationsResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetOptionExpirationsResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_lookup_option_symbols_response_from_json(
            response in any::<LookupOptionSymbolsResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<LookupOptionSymbolsResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_historical_quotes_response_from_json(
            response in any::<GetHistoricalQuotesResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetHistoricalQuotesResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_timesales_response_from_json(
            response in any::<GetTimeAndSalesResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetTimeAndSalesResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_etb_securities_response_from_json(
            response in any::<GetEtbSecuritiesResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetEtbSecuritiesResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_clock_response_from_json(response in any::<GetClockResponseWire>()) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetClockResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_calendar_response_from_json(
            response in any::<GetCalendarResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<GetCalendarResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_search_response_from_json(
            response in any::<SearchCompaniesResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<SearchCompaniesResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_deserialize_lookup_symbol_response_from_json(
            response in any::<LookupSymbolResponseWire>()
        ) {
            let json = serde_json::to_string_pretty(&response).expect("serialize");
            let result: std::result::Result<LookupSymbolResponse, serde_json::Error> =
                serde_json::from_str(&json);
            assert!(result.is_ok());
        }
    }
}
