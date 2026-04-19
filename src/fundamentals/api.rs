//! Traits exposing the Fundamentals (beta) REST endpoints, in both blocking
//! and non-blocking flavors.
//!
//! Upstream documentation:
//! <https://documentation.tradier.com/brokerage-api/markets/get-company>.

use crate::fundamentals::types::{
    CompanyResponse, CorporateActionResponse, CorporateCalendarResponse, DividendResponse,
    FinancialsResponse, RatiosResponse, StatisticsResponse, Symbol,
};
use crate::{error::Result, utils::Sealed};

pub mod non_blocking {
    use super::*;

    /// The non-blocking (async) surface of the Tradier Fundamentals (beta)
    /// REST API. Every method accepts a slice of [`Symbol`]s and returns a
    /// vector of endpoint-specific response envelopes — one per requested
    /// symbol.
    #[async_trait::async_trait]
    pub trait Fundamentals: Sealed {
        /// `GET /beta/markets/fundamentals/company` — company profile and
        /// share-class details for one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_company(&self, symbols: &[Symbol]) -> Result<Vec<CompanyResponse>>;

        /// `GET /beta/markets/fundamentals/corporate_calendars` — corporate
        /// event calendars (earnings, IPOs, splits, ...) for one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_corporate_calendars(
            &self,
            symbols: &[Symbol],
        ) -> Result<Vec<CorporateCalendarResponse>>;

        /// `GET /beta/markets/fundamentals/dividends` — dividend history for
        /// one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_dividends(&self, symbols: &[Symbol]) -> Result<Vec<DividendResponse>>;

        /// `GET /beta/markets/fundamentals/corporate_actions` — corporate
        /// actions (splits, mergers, spinoffs) for one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_corporate_actions(
            &self,
            symbols: &[Symbol],
        ) -> Result<Vec<CorporateActionResponse>>;

        /// `GET /beta/markets/fundamentals/ratios` — financial ratios (P/E,
        /// EPS, margins) for one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_ratios(&self, symbols: &[Symbol]) -> Result<Vec<RatiosResponse>>;

        /// `GET /beta/markets/fundamentals/financials` — quarterly / annual
        /// financial reports (balance sheet, income, cash flow) for one or
        /// more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_financials(&self, symbols: &[Symbol]) -> Result<Vec<FinancialsResponse>>;

        /// `GET /beta/markets/fundamentals/statistics` — price statistics
        /// (averages, volatility, trailing returns) for one or more symbols.
        ///
        /// # Errors
        /// Returns [`crate::Error::NetworkError`] on transport / HTTP failures.
        async fn get_statistics(&self, symbols: &[Symbol]) -> Result<Vec<StatisticsResponse>>;
    }
}

pub mod blocking {
    use super::*;

    /// The blocking surface of the Tradier Fundamentals (beta) REST API.
    pub trait Fundamentals: Sealed {
        /// See [`super::non_blocking::Fundamentals::get_company`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_company`].
        fn get_company(&self, symbols: &[Symbol]) -> Result<Vec<CompanyResponse>>;

        /// See [`super::non_blocking::Fundamentals::get_corporate_calendars`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_corporate_calendars`].
        fn get_corporate_calendars(
            &self,
            symbols: &[Symbol],
        ) -> Result<Vec<CorporateCalendarResponse>>;

        /// See [`super::non_blocking::Fundamentals::get_dividends`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_dividends`].
        fn get_dividends(&self, symbols: &[Symbol]) -> Result<Vec<DividendResponse>>;

        /// See [`super::non_blocking::Fundamentals::get_corporate_actions`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_corporate_actions`].
        fn get_corporate_actions(
            &self,
            symbols: &[Symbol],
        ) -> Result<Vec<CorporateActionResponse>>;

        /// See [`super::non_blocking::Fundamentals::get_ratios`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_ratios`].
        fn get_ratios(&self, symbols: &[Symbol]) -> Result<Vec<RatiosResponse>>;

        /// See [`super::non_blocking::Fundamentals::get_financials`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_financials`].
        fn get_financials(&self, symbols: &[Symbol]) -> Result<Vec<FinancialsResponse>>;

        /// See [`super::non_blocking::Fundamentals::get_statistics`].
        ///
        /// # Errors
        /// See [`super::non_blocking::Fundamentals::get_statistics`].
        fn get_statistics(&self, symbols: &[Symbol]) -> Result<Vec<StatisticsResponse>>;
    }
}
