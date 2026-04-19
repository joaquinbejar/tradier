//! Traits exposing the Market Data REST endpoints, in both blocking and
//! non-blocking flavors.
//!
//! Upstream documentation: <https://documentation.tradier.com/brokerage-api/markets/>.

use chrono::{DateTime, NaiveDate, Utc};

use crate::market_data::types::{
    CalendarMonth, CalendarYear, DelayedFlag, Exchanges, GetCalendarResponse, GetClockResponse,
    GetEtbSecuritiesResponse, GetHistoricalQuotesResponse, GetOptionChainsResponse,
    GetOptionExpirationsResponse, GetOptionStrikesResponse, GetQuotesResponse,
    GetTimeAndSalesResponse, Greeks, HistoryInterval, IncludeAllRoots, IncludeStrikes,
    IndexesFlag, LookupOptionSymbolsResponse, LookupSymbolResponse, SearchCompaniesResponse,
    SecurityTypes, SessionFilter, Symbol, Symbols, TimeSalesInterval,
};
use crate::{error::Result, utils::Sealed};

pub mod non_blocking {
    use super::*;

    /// The non-blocking (async) surface of the Tradier Market Data REST API.
    #[async_trait::async_trait]
    pub trait MarketData: Sealed {
        /// `GET /v1/markets/quotes` — retrieve quotes for one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_quotes(
            &self,
            symbols: &Symbols,
            greeks: Option<Greeks>,
        ) -> Result<GetQuotesResponse>;

        /// `POST /v1/markets/quotes` — form-encoded variant for large symbol lists.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn post_quotes(
            &self,
            symbols: &Symbols,
            greeks: Option<Greeks>,
        ) -> Result<GetQuotesResponse>;

        /// `GET /v1/markets/options/chains` — option chain for a symbol + expiration.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_option_chains(
            &self,
            symbol: &Symbol,
            expiration: &NaiveDate,
            greeks: Option<Greeks>,
        ) -> Result<GetOptionChainsResponse>;

        /// `GET /v1/markets/options/strikes` — list of strikes for a given expiration.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_option_strikes(
            &self,
            symbol: &Symbol,
            expiration: &NaiveDate,
        ) -> Result<GetOptionStrikesResponse>;

        /// `GET /v1/markets/options/expirations` — list of available expirations.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_option_expirations(
            &self,
            symbol: &Symbol,
            include_all_roots: Option<IncludeAllRoots>,
            strikes: Option<IncludeStrikes>,
        ) -> Result<GetOptionExpirationsResponse>;

        /// `GET /v1/markets/options/lookup` — list option root symbols for an underlying.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn lookup_option_symbols(
            &self,
            underlying: &Symbol,
        ) -> Result<LookupOptionSymbolsResponse>;

        /// `GET /v1/markets/history` — historical OHLCV bars.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_historical_quotes(
            &self,
            symbol: &Symbol,
            interval: Option<HistoryInterval>,
            start: Option<&NaiveDate>,
            end: Option<&NaiveDate>,
            session_filter: Option<SessionFilter>,
        ) -> Result<GetHistoricalQuotesResponse>;

        /// `GET /v1/markets/timesales` — intraday time-and-sales data.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_time_and_sales(
            &self,
            symbol: &Symbol,
            interval: Option<TimeSalesInterval>,
            start: Option<&DateTime<Utc>>,
            end: Option<&DateTime<Utc>>,
            session_filter: Option<SessionFilter>,
        ) -> Result<GetTimeAndSalesResponse>;

        /// `GET /v1/markets/etb` — easy-to-borrow securities list.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_etb_securities(&self) -> Result<GetEtbSecuritiesResponse>;

        /// `GET /v1/markets/clock` — market clock state.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_clock(&self, delayed: Option<DelayedFlag>) -> Result<GetClockResponse>;

        /// `GET /v1/markets/calendar` — market calendar for a given month.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_calendar(
            &self,
            month: Option<CalendarMonth>,
            year: Option<CalendarYear>,
        ) -> Result<GetCalendarResponse>;

        /// `GET /v1/markets/search` — search for companies.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn search_companies(
            &self,
            q: &str,
            indexes: Option<IndexesFlag>,
        ) -> Result<SearchCompaniesResponse>;

        /// `GET /v1/markets/lookup` — look up a symbol.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn lookup_symbol(
            &self,
            q: &str,
            exchanges: Option<&Exchanges>,
            types: Option<&SecurityTypes>,
        ) -> Result<LookupSymbolResponse>;
    }
}

pub mod blocking {
    use super::*;

    /// The blocking surface of the Tradier Market Data REST API.
    pub trait MarketData: Sealed {
        /// See [`super::non_blocking::MarketData::get_quotes`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_quotes`].
        fn get_quotes(
            &self,
            symbols: &Symbols,
            greeks: Option<Greeks>,
        ) -> Result<GetQuotesResponse>;

        /// See [`super::non_blocking::MarketData::post_quotes`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::post_quotes`].
        fn post_quotes(
            &self,
            symbols: &Symbols,
            greeks: Option<Greeks>,
        ) -> Result<GetQuotesResponse>;

        /// See [`super::non_blocking::MarketData::get_option_chains`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_option_chains`].
        fn get_option_chains(
            &self,
            symbol: &Symbol,
            expiration: &NaiveDate,
            greeks: Option<Greeks>,
        ) -> Result<GetOptionChainsResponse>;

        /// See [`super::non_blocking::MarketData::get_option_strikes`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_option_strikes`].
        fn get_option_strikes(
            &self,
            symbol: &Symbol,
            expiration: &NaiveDate,
        ) -> Result<GetOptionStrikesResponse>;

        /// See [`super::non_blocking::MarketData::get_option_expirations`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_option_expirations`].
        fn get_option_expirations(
            &self,
            symbol: &Symbol,
            include_all_roots: Option<IncludeAllRoots>,
            strikes: Option<IncludeStrikes>,
        ) -> Result<GetOptionExpirationsResponse>;

        /// See [`super::non_blocking::MarketData::lookup_option_symbols`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::lookup_option_symbols`].
        fn lookup_option_symbols(
            &self,
            underlying: &Symbol,
        ) -> Result<LookupOptionSymbolsResponse>;

        /// See [`super::non_blocking::MarketData::get_historical_quotes`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_historical_quotes`].
        fn get_historical_quotes(
            &self,
            symbol: &Symbol,
            interval: Option<HistoryInterval>,
            start: Option<&NaiveDate>,
            end: Option<&NaiveDate>,
            session_filter: Option<SessionFilter>,
        ) -> Result<GetHistoricalQuotesResponse>;

        /// See [`super::non_blocking::MarketData::get_time_and_sales`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_time_and_sales`].
        fn get_time_and_sales(
            &self,
            symbol: &Symbol,
            interval: Option<TimeSalesInterval>,
            start: Option<&DateTime<Utc>>,
            end: Option<&DateTime<Utc>>,
            session_filter: Option<SessionFilter>,
        ) -> Result<GetTimeAndSalesResponse>;

        /// See [`super::non_blocking::MarketData::get_etb_securities`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_etb_securities`].
        fn get_etb_securities(&self) -> Result<GetEtbSecuritiesResponse>;

        /// See [`super::non_blocking::MarketData::get_clock`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_clock`].
        fn get_clock(&self, delayed: Option<DelayedFlag>) -> Result<GetClockResponse>;

        /// See [`super::non_blocking::MarketData::get_calendar`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::get_calendar`].
        fn get_calendar(
            &self,
            month: Option<CalendarMonth>,
            year: Option<CalendarYear>,
        ) -> Result<GetCalendarResponse>;

        /// See [`super::non_blocking::MarketData::search_companies`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::search_companies`].
        fn search_companies(
            &self,
            q: &str,
            indexes: Option<IndexesFlag>,
        ) -> Result<SearchCompaniesResponse>;

        /// See [`super::non_blocking::MarketData::lookup_symbol`].
        ///
        /// # Errors
        /// See [`super::non_blocking::MarketData::lookup_symbol`].
        fn lookup_symbol(
            &self,
            q: &str,
            exchanges: Option<&Exchanges>,
            types: Option<&SecurityTypes>,
        ) -> Result<LookupSymbolResponse>;
    }
}
