//! Blocking client wrapper for the asynchronous Tradier REST client.
//!
//! This type provides a *synchronous* API surface over the async client,
//! intended for binaries, CLIs, and unit tests that do not run a Tokio
//! runtime. It is **not** meant to be constructed or used while a Tokio
//! runtime is active; use the async client in async contexts.
//!
//! # Design
//! - The blocking client owns a private, single-thread Tokio runtime used
//!   exclusively when no other runtime is active.
//! - Construction and use *inside* an existing Tokio runtime are disallowed
//!   to avoid nested event loops and drop-in-runtime pitfalls.
//!
//! # When to use
//! - You want to make blocking, request–response calls (e.g., demo code,
//!   simple scripts, integration tests running in a plain `fn main()`).
//! - You are **not** inside a Tokio runtime (no `#[tokio::main]`, no async tests).
//!
//! # When **not** to use
//! - Any code already running under Tokio (e.g., `#[tokio::main]`, `#[tokio::test]`).
//!   In those cases, import and call the async client directly.
use chrono::{DateTime, NaiveDate, Utc};
use tokio::runtime::{Handle, Runtime};

use crate::{
    accounts::types::{
        AccountNumber, EventType, GainLossSortBy, GetAccountBalancesResponse,
        GetAccountGainLossResponse, GetAccountHistoryResponse, GetAccountOrdersResponse,
        IncludeTags, Limit, Page,
    },
    accounts::{api::blocking::Accounts, api::non_blocking::Accounts as NonBlockingAccounts},
    client::non_blocking::TradierRestClient as AsyncClient,
    common::SortOrder,
    fundamentals::{
        api::blocking::Fundamentals,
        api::non_blocking::Fundamentals as NonBlockingFundamentals,
        types::{
            CompanyResponse, CorporateActionResponse, CorporateCalendarResponse, DividendResponse,
            FinancialsResponse, RatiosResponse, StatisticsResponse,
        },
    },
    market_data::{
        api::blocking::MarketData,
        api::non_blocking::MarketData as NonBlockingMarketData,
        types::{
            CalendarMonth, CalendarYear, DelayedFlag, Exchanges, GetCalendarResponse,
            GetClockResponse, GetEtbSecuritiesResponse, GetHistoricalQuotesResponse,
            GetOptionChainsResponse, GetOptionExpirationsResponse, GetOptionStrikesResponse,
            GetQuotesResponse, GetTimeAndSalesResponse, Greeks, HistoryInterval, IncludeAllRoots,
            IncludeStrikes, IndexesFlag, LookupOptionSymbolsResponse, LookupSymbolResponse,
            SearchCompaniesResponse, SecurityTypes, SessionFilter, Symbol, Symbols,
            TimeSalesInterval,
        },
    },
    user::{api::blocking::User, api::non_blocking::User as NonBlockingUser, UserProfileResponse},
    utils::Sealed,
    Config, Result,
};

/// A synchronous façade over [`AsyncClient`] for environments without a Tokio runtime.
///
/// This client is intended for blocking use cases. It will **refuse** to be
/// constructed or used while a Tokio runtime is active to prevent nested
/// runtimes and subtle shutdown bugs.
///
/// # Examples
///
/// Basic blocking usage:
/// ```no_run
/// use tradier::{blocking::Client, blocking::operation::User, Config};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // No tokio runtime here
///     let cfg = Config::new();
///     let client = Client::new(cfg)?;
///     let me = client.get_user_profile()?;
///     println!("{:?}", me);
///     Ok(())
/// }
/// ```
///
/// Attempting to construct inside a Tokio runtime returns an error:
/// ```no_run
/// # use tradier::{blocking::Client, Config};
/// # use tradier::Error;
/// #[tokio::main]
/// async fn main() {
///     let cfg = Config::new();
///     let err = Client::new(cfg).unwrap_err();
///     // You can match your error variant if exposed as public API:
///     // assert!(matches!(err, Error::BlockingInsideRuntime));
///     let _ = err; // documentation only
/// }
/// ```
#[derive(Debug)]
pub struct BlockingTradierRestClient {
    rest_client: AsyncClient,
    /// Private single-thread runtime for blocking operations when no external
    /// runtime exists. Never used if a Tokio runtime is currently active,
    /// because construction/usage in that state is rejected.
    runtime: Runtime,
}

impl BlockingTradierRestClient {
    /// Constructs a blocking client for use **outside** of a Tokio runtime.
    ///
    /// If a Tokio runtime is currently active on the calling thread, this
    /// returns [`Error::BlockingInsideRuntime`] (or your equivalent error type).
    ///
    /// # Errors
    /// - [`Error::BlockingInsideRuntime`]: a runtime is already active.
    /// - Any error that arises while building the internal single-thread runtime.
    pub fn new(config: Config) -> Result<Self> {
        if Handle::try_current().is_ok() {
            return Err(crate::Error::BlockingClientInsideAsyncRuntime);
        }

        Ok(Self {
            rest_client: AsyncClient::new(config),
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?,
        })
    }
}

impl Sealed for BlockingTradierRestClient {}

impl User for BlockingTradierRestClient {
    fn get_user_profile(&self) -> Result<UserProfileResponse> {
        self.runtime.block_on(self.rest_client.get_user_profile())
    }
}

impl Accounts for BlockingTradierRestClient {
    fn get_account_balances(
        &self,
        account_number: &AccountNumber,
    ) -> Result<GetAccountBalancesResponse> {
        self.runtime
            .block_on(self.rest_client.get_account_balances(account_number))
    }

    fn get_account_positions(
        &self,
        account_number: &AccountNumber,
    ) -> Result<crate::types::GetAccountPositionsResponse> {
        self.runtime
            .block_on(self.rest_client.get_account_positions(account_number))
    }

    fn get_account_history(
        &self,
        account_number: &AccountNumber,
        page: Option<Page>,
        limit: Option<Limit>,
        event_type: Option<EventType>,
    ) -> Result<GetAccountHistoryResponse> {
        self.runtime.block_on(self.rest_client.get_account_history(
            account_number,
            page,
            limit,
            event_type,
        ))
    }

    fn get_account_gain_loss(
        &self,
        account_number: &AccountNumber,
        page: Option<Page>,
        limit: Option<Limit>,
        sort_by: Option<GainLossSortBy>,
        sort_order: Option<SortOrder>,
    ) -> Result<GetAccountGainLossResponse> {
        self.runtime
            .block_on(self.rest_client.get_account_gain_loss(
                account_number,
                page,
                limit,
                sort_by,
                sort_order,
            ))
    }

    fn get_account_orders(
        &self,
        account_number: &AccountNumber,
        page: &Page,
        limit: &Limit,
        include_tags: &IncludeTags,
    ) -> Result<GetAccountOrdersResponse> {
        self.runtime.block_on(self.rest_client.get_account_orders(
            account_number,
            page,
            limit,
            include_tags,
        ))
    }
}

impl MarketData for BlockingTradierRestClient {
    fn get_quotes(&self, symbols: &Symbols, greeks: Option<Greeks>) -> Result<GetQuotesResponse> {
        self.runtime
            .block_on(self.rest_client.get_quotes(symbols, greeks))
    }

    fn post_quotes(&self, symbols: &Symbols, greeks: Option<Greeks>) -> Result<GetQuotesResponse> {
        self.runtime
            .block_on(self.rest_client.post_quotes(symbols, greeks))
    }

    fn get_option_chains(
        &self,
        symbol: &Symbol,
        expiration: &NaiveDate,
        greeks: Option<Greeks>,
    ) -> Result<GetOptionChainsResponse> {
        self.runtime.block_on(
            self.rest_client
                .get_option_chains(symbol, expiration, greeks),
        )
    }

    fn get_option_strikes(
        &self,
        symbol: &Symbol,
        expiration: &NaiveDate,
    ) -> Result<GetOptionStrikesResponse> {
        self.runtime
            .block_on(self.rest_client.get_option_strikes(symbol, expiration))
    }

    fn get_option_expirations(
        &self,
        symbol: &Symbol,
        include_all_roots: Option<IncludeAllRoots>,
        strikes: Option<IncludeStrikes>,
    ) -> Result<GetOptionExpirationsResponse> {
        self.runtime
            .block_on(
                self.rest_client
                    .get_option_expirations(symbol, include_all_roots, strikes),
            )
    }

    fn lookup_option_symbols(&self, underlying: &Symbol) -> Result<LookupOptionSymbolsResponse> {
        self.runtime
            .block_on(self.rest_client.lookup_option_symbols(underlying))
    }

    fn get_historical_quotes(
        &self,
        symbol: &Symbol,
        interval: Option<HistoryInterval>,
        start: Option<&NaiveDate>,
        end: Option<&NaiveDate>,
        session_filter: Option<SessionFilter>,
    ) -> Result<GetHistoricalQuotesResponse> {
        self.runtime
            .block_on(self.rest_client.get_historical_quotes(
                symbol,
                interval,
                start,
                end,
                session_filter,
            ))
    }

    fn get_time_and_sales(
        &self,
        symbol: &Symbol,
        interval: Option<TimeSalesInterval>,
        start: Option<&DateTime<Utc>>,
        end: Option<&DateTime<Utc>>,
        session_filter: Option<SessionFilter>,
    ) -> Result<GetTimeAndSalesResponse> {
        self.runtime.block_on(self.rest_client.get_time_and_sales(
            symbol,
            interval,
            start,
            end,
            session_filter,
        ))
    }

    fn get_etb_securities(&self) -> Result<GetEtbSecuritiesResponse> {
        self.runtime.block_on(self.rest_client.get_etb_securities())
    }

    fn get_clock(&self, delayed: Option<DelayedFlag>) -> Result<GetClockResponse> {
        self.runtime.block_on(self.rest_client.get_clock(delayed))
    }

    fn get_calendar(
        &self,
        month: Option<CalendarMonth>,
        year: Option<CalendarYear>,
    ) -> Result<GetCalendarResponse> {
        self.runtime
            .block_on(self.rest_client.get_calendar(month, year))
    }

    fn search_companies(
        &self,
        q: &str,
        indexes: Option<IndexesFlag>,
    ) -> Result<SearchCompaniesResponse> {
        self.runtime
            .block_on(self.rest_client.search_companies(q, indexes))
    }

    fn lookup_symbol(
        &self,
        q: &str,
        exchanges: Option<&Exchanges>,
        types: Option<&SecurityTypes>,
    ) -> Result<LookupSymbolResponse> {
        self.runtime
            .block_on(self.rest_client.lookup_symbol(q, exchanges, types))
    }
}

impl Fundamentals for BlockingTradierRestClient {
    fn get_company(&self, symbols: &[Symbol]) -> Result<Vec<CompanyResponse>> {
        self.runtime
            .block_on(self.rest_client.get_company(symbols))
    }

    fn get_corporate_calendars(
        &self,
        symbols: &[Symbol],
    ) -> Result<Vec<CorporateCalendarResponse>> {
        self.runtime
            .block_on(self.rest_client.get_corporate_calendars(symbols))
    }

    fn get_dividends(&self, symbols: &[Symbol]) -> Result<Vec<DividendResponse>> {
        self.runtime
            .block_on(self.rest_client.get_dividends(symbols))
    }

    fn get_corporate_actions(
        &self,
        symbols: &[Symbol],
    ) -> Result<Vec<CorporateActionResponse>> {
        self.runtime
            .block_on(self.rest_client.get_corporate_actions(symbols))
    }

    fn get_ratios(&self, symbols: &[Symbol]) -> Result<Vec<RatiosResponse>> {
        self.runtime
            .block_on(self.rest_client.get_ratios(symbols))
    }

    fn get_financials(&self, symbols: &[Symbol]) -> Result<Vec<FinancialsResponse>> {
        self.runtime
            .block_on(self.rest_client.get_financials(symbols))
    }

    fn get_statistics(&self, symbols: &[Symbol]) -> Result<Vec<StatisticsResponse>> {
        self.runtime
            .block_on(self.rest_client.get_statistics(symbols))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;

    use crate::{
        accounts::test_support::{
            GetAccountBalancesResponseWire, GetAccountGainLossResponseWire,
            GetAccountHistoryResponseWire, GetAccountOrdersResponseWire,
            GetAccountPositionsResponseWire,
        },
        accounts::types::{EventType, GainLossSortBy, Limit, Page},
        common::SortOrder,
        user::test_support::GetUserProfileResponseWire,
        utils::tests::with_env_vars,
        Config,
    };

    use httpmock::MockServer;
    use proptest::prelude::*;

    fn run_gain_loss_proptest(
        server: &RefCell<MockServer>,
        expected_query_params: &[(&str, &str)],
        page: Option<Page>,
        limit: Option<Limit>,
        sort_by: Option<GainLossSortBy>,
        sort_order: Option<SortOrder>,
    ) {
        proptest!(|(response in any::<GetAccountGainLossResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                let mut when = when
                    .path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/gainloss"))).unwrap().path())
                    .header("accept", "application/json");
                for (k, v) in expected_query_params {
                    when = when.query_param(*k, *v);
                }
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response).expect("serialization to work"));
            });
            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_gain_loss(
                    &ascii_string.parse().expect("valid ascii"),
                    page.clone(),
                    limit.clone(),
                    sort_by.clone(),
                    sort_order.clone(),
                );
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });
    }

    #[test]
    fn test_blocking_client() {
        let server = MockServer::start();
        let server = RefCell::new(server);

        // Test GetUserProfile
        // Low case count: these tests exercise HTTP wiring, not domain logic.
        // Domain correctness is verified by the deserialization and schema proptests.

        proptest!(ProptestConfig::with_cases(16), |(response in any::<GetUserProfileResponseWire>())| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url("/v1/user/profile")).unwrap().path())
                    .header("accept", "application/json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_user_profile();
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });

        // Test GetAccountBalances

        proptest!(ProptestConfig::with_cases(16), |(response in any::<GetAccountBalancesResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/balances"))).unwrap().path())
                    .header("accept", "application/json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_balances(&ascii_string.parse().expect("valid ascii"));
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });

        // Test GetAccountPositions

        proptest!(ProptestConfig::with_cases(16), |(response in any::<GetAccountPositionsResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/positions"))).unwrap().path())
                    .header("accept", "application/json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_positions(&ascii_string.parse().expect("valid ascii"));
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });

        // Test GetAccountHistory (no query params)

        proptest!(|(response in any::<GetAccountHistoryResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/history"))).unwrap().path())
                    .header("accept", "application/json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_history(
                    &ascii_string.parse().expect("valid ascii"),
                    None,
                    None,
                    None,
                );
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });

        // Test GetAccountHistory (with query params)

        proptest!(|(response in any::<GetAccountHistoryResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/history"))).unwrap().path())
                    .header("accept", "application/json")
                    .query_param("page", "2")
                    .query_param("limit", "50")
                    .query_param("type", "trade");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_history(
                    &ascii_string.parse().expect("valid ascii"),
                    Some(Page::new(2)),
                    Some(Limit::new(50)),
                    Some(EventType::Trade),
                );
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });

        run_gain_loss_proptest(&server, &[], None, None, None, None);

        run_gain_loss_proptest(
            &server,
            &[
                ("page", "2"),
                ("limit", "50"),
                ("sortBy", "symbol"),
                ("sort", "asc"),
            ],
            Some(Page::new(2)),
            Some(Limit::new(50)),
            Some(GainLossSortBy::Symbol),
            Some(SortOrder::Asc),
        );

        run_gain_loss_proptest(
            &server,
            &[("sortBy", "closedate"), ("sort", "desc")],
            None,
            None,
            Some(GainLossSortBy::CloseDate),
            Some(SortOrder::Desc),
        );

        run_gain_loss_proptest(
            &server,
            &[("sortBy", "opendate")],
            None,
            None,
            Some(GainLossSortBy::OpenDate),
            None,
        );

        run_gain_loss_proptest(
            &server,
            &[("sortBy", "gainloss")],
            None,
            None,
            Some(GainLossSortBy::GainLoss),
            None,
        );

        // Test GetAccountOrders with includeTags=true
        // Reduced case count: GetAccountOrdersResponseWire is deeply nested
        // and the default 256 cases exceed tarpaulin's timeout budget.

        proptest!(ProptestConfig::with_cases(16), |(response in any::<GetAccountOrdersResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let page = Page::new(3);
            let limit = Limit::new(40);
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/orders"))).unwrap().path())
                    .header("accept", "application/json")
                    .query_param("page", page.to_string())
                    .query_param("limit", limit.to_string())
                    .query_param("includeTags", "true");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_orders(
                    &ascii_string.parse().expect("valid ascii"),
                    &page,
                    &limit,
                    &IncludeTags::from(true),
                );
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });

        // Test GetAccountOrders with includeTags=false

        proptest!(ProptestConfig::with_cases(16), |(response in any::<GetAccountOrdersResponseWire>(),
                ascii_string in prop::collection::vec(0x20u8..0x7fu8, 1..256)
            .prop_flat_map(|vec| {
                Just(vec.into_iter().map(|c| c as char).collect::<String>())
            })
            .prop_filter("Strings must not be empty or blank", |v| !v.trim().is_empty()))| {
            let server = server.borrow_mut();
            let page = Page::default();
            let limit = Limit::default();
            let mut operation = server.mock(|when, then| {
                when.path(url::Url::parse(&server.url(format!("/v1/accounts/{ascii_string}/orders"))).unwrap().path())
                    .header("accept", "application/json")
                    .query_param("page", page.to_string())
                    .query_param("limit", limit.to_string())
                    .query_param("includeTags", "false");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = BlockingTradierRestClient::new(config).expect("client to initialize");
                let response = sut.get_account_orders(
                    &ascii_string.parse().expect("valid ascii"),
                    &page,
                    &limit,
                    &IncludeTags::from(false),
                );
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });
    }

    #[tokio::test]
    async fn test_should_not_be_able_to_create_within_an_async_runtime() {
        let config = Config::new();
        let sut = BlockingTradierRestClient::new(config);
        assert!(sut.is_err());
    }
}

#[cfg(test)]
mod market_data_tests {
    use super::BlockingTradierRestClient;
    use crate::{
        market_data::{
            api::blocking::MarketData,
            test_support::{
                GetCalendarResponseWire, GetClockResponseWire, GetEtbSecuritiesResponseWire,
                GetHistoricalQuotesResponseWire, GetOptionChainsResponseWire,
                GetOptionExpirationsResponseWire, GetOptionStrikesResponseWire,
                GetQuotesResponseWire, GetTimeAndSalesResponseWire,
                LookupOptionSymbolsResponseWire, LookupSymbolResponseWire,
                SearchCompaniesResponseWire,
            },
            types::{
                CalendarMonth, CalendarYear, DelayedFlag, Exchanges, Greeks, HistoryInterval,
                IncludeAllRoots, IncludeStrikes, IndexesFlag, SecurityTypes, SessionFilter, Symbol,
                Symbols, TimeSalesInterval,
            },
        },
        utils::tests::with_env_vars,
        Config,
    };
    use chrono::{NaiveDate, TimeZone, Utc};
    use httpmock::MockServer;
    use proptest::prelude::*;
    use std::cell::RefCell;

    fn make_symbol(s: &str) -> Symbol {
        s.parse().expect("valid symbol")
    }

    fn make_symbols(syms: &[&str]) -> Symbols {
        Symbols::new(syms.iter().map(|s| make_symbol(s)))
    }

    fn make_client(server: &MockServer) -> BlockingTradierRestClient {
        let mut config = Config::new();
        // Swap the base URL to point at the mock server.
        config.rest_api.base_url = server.base_url();
        BlockingTradierRestClient::new(config).expect("client to initialize")
    }

    fn run_with_env<F: FnOnce()>(server: &MockServer, f: F) {
        with_env_vars(
            vec![
                ("TRADIER_REST_BASE_URL", &server.base_url()),
                ("TRADIER_ACCESS_TOKEN", "testToken"),
            ],
            f,
        );
    }

    // 1. GET /v1/markets/quotes ------------------------------------------------

    #[test]
    fn test_get_quotes_happy_path_returns_decoded_struct() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetQuotesResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/v1/markets/quotes")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL,MSFT")
                        .query_param("greeks", "true");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_quotes(
                        &make_symbols(&["AAPL", "MSFT"]),
                        Some(Greeks::new(true)),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_quotes_without_greeks_omits_query_param() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/v1/markets/quotes")
                .query_param("symbols", "AAPL")
                .is_true(|req| req.query_params().iter().all(|(k, _)| k != "greeks"));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"quotes":{}}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_quotes(&make_symbols(&["AAPL"]), None);
            assert!(resp.is_ok());
            op.assert();
        });
    }

    #[test]
    fn test_get_quotes_server_error_surfaces_network_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/quotes");
            then.status(500)
                .header("content-type", "application/json")
                .body("not json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_quotes(&make_symbols(&["AAPL"]), None);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_quotes_malformed_body_surfaces_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/quotes");
            then.status(200)
                .header("content-type", "application/json")
                .body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_quotes(&make_symbols(&["AAPL"]), None);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_quotes_four_hundred_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/quotes");
            then.status(400)
                .header("content-type", "application/json")
                .body(r#"{"fault":{"faultstring":"bad"}}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_quotes(&make_symbols(&["AAPL"]), None);
            // The client deserializes JSON bodies even on non-2xx because reqwest
            // does not automatically raise on status. The body here is valid JSON
            // but does not match GetQuotesResponse, so deserialization must fail.
            assert!(resp.is_err());
            op.assert();
        });
    }

    // 2. POST /v1/markets/quotes ----------------------------------------------

    #[test]
    fn test_post_quotes_happy_path_posts_form_body() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/markets/quotes")
                .header("accept", "application/json")
                .body_includes("symbols=AAPL%2CMSFT")
                .body_includes("greeks=true");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"quotes":{}}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp =
                client.post_quotes(&make_symbols(&["AAPL", "MSFT"]), Some(Greeks::new(true)));
            assert!(resp.is_ok());
            op.assert();
        });
    }

    #[test]
    fn test_post_quotes_without_greeks_form_body_has_no_greeks_field() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/markets/quotes")
                .body_includes("symbols=AAPL")
                .is_true(|req| !req.body_string().contains("greeks"));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"quotes":{}}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.post_quotes(&make_symbols(&["AAPL"]), None);
            assert!(resp.is_ok());
            op.assert();
        });
    }

    // 3. GET /v1/markets/options/chains ---------------------------------------

    #[test]
    fn test_get_option_chains_happy_path_verifies_query_params() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetOptionChainsResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/v1/markets/options/chains")
                        .query_param("symbol", "AAPL")
                        .query_param("expiration", "2024-06-21")
                        .query_param("greeks", "false");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let expiration = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();
                    let resp = client.get_option_chains(
                        &make_symbol("AAPL"),
                        &expiration,
                        Some(Greeks::new(false)),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_option_chains_server_error_surfaces_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/options/chains");
            then.status(503);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let expiration = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();
            let resp = client.get_option_chains(&make_symbol("AAPL"), &expiration, None);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // 4. GET /v1/markets/options/strikes --------------------------------------

    #[test]
    fn test_get_option_strikes_happy_path_returns_decoded_struct() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetOptionStrikesResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/options/strikes")
                        .query_param("symbol", "AAPL")
                        .query_param("expiration", "2024-06-21");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let expiration = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();
                    let resp = client
                        .get_option_strikes(&make_symbol("AAPL"), &expiration);
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    // 5. GET /v1/markets/options/expirations ----------------------------------

    #[test]
    fn test_get_option_expirations_happy_path_verifies_query_params() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetOptionExpirationsResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/options/expirations")
                        .query_param("symbol", "AAPL")
                        .query_param("includeAllRoots", "true")
                        .query_param("strikes", "true");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_option_expirations(
                        &make_symbol("AAPL"),
                        Some(IncludeAllRoots::new(true)),
                        Some(IncludeStrikes::new(true)),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    // 6. GET /v1/markets/options/lookup ---------------------------------------

    #[test]
    fn test_lookup_option_symbols_happy_path_returns_decoded_struct() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<LookupOptionSymbolsResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/options/lookup")
                        .query_param("underlying", "AAPL");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.lookup_option_symbols(&make_symbol("AAPL"));
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    // 7. GET /v1/markets/history ----------------------------------------------

    #[test]
    fn test_get_historical_quotes_happy_path_verifies_all_query_params() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetHistoricalQuotesResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/history")
                        .query_param("symbol", "AAPL")
                        .query_param("interval", "daily")
                        .query_param("start", "2024-01-01")
                        .query_param("end", "2024-01-31")
                        .query_param("session_filter", "open");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
                    let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
                    let resp = client.get_historical_quotes(
                        &make_symbol("AAPL"),
                        Some(HistoryInterval::Daily),
                        Some(&start),
                        Some(&end),
                        Some(SessionFilter::Open),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_historical_quotes_with_no_options_sends_only_symbol() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/history")
                .query_param("symbol", "AAPL")
                .is_true(|req| {
                    req.query_params()
                        .iter()
                        .filter(|(k, _)| k != "symbol")
                        .count()
                        == 0
                });
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"history": null}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_historical_quotes(&make_symbol("AAPL"), None, None, None, None);
            assert!(resp.is_ok());
            op.assert();
        });
    }

    // 8. GET /v1/markets/timesales --------------------------------------------

    #[test]
    fn test_get_time_and_sales_happy_path_verifies_all_query_params() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetTimeAndSalesResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/timesales")
                        .query_param("symbol", "AAPL")
                        .query_param("interval", "5min")
                        .query_param("start", "2024-01-15 09:30")
                        .query_param("end", "2024-01-15 16:00")
                        .query_param("session_filter", "all");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let start = Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, 0).unwrap();
                    let end = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
                    let resp = client.get_time_and_sales(
                        &make_symbol("AAPL"),
                        Some(TimeSalesInterval::FiveMinutes),
                        Some(&start),
                        Some(&end),
                        Some(SessionFilter::All),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    // 9. GET /v1/markets/etb --------------------------------------------------

    #[test]
    fn test_get_etb_securities_happy_path_returns_decoded_struct() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetEtbSecuritiesResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/etb");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_etb_securities();
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_etb_securities_not_authorized_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/etb");
            then.status(401)
                .header("content-type", "application/json")
                .body("\"not a json object\"");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_etb_securities();
            assert!(resp.is_err());
            op.assert();
        });
    }

    // 10. GET /v1/markets/clock -----------------------------------------------

    #[test]
    fn test_get_clock_happy_path_verifies_delayed_query_param() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetClockResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/clock")
                        .query_param("delayed", "true");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_clock(Some(DelayedFlag::new(true)));
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_clock_without_delayed_omits_query_param() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/clock")
                .is_true(|req| req.query_params().iter().all(|(k, _)| k != "delayed"));
            then.status(200).header("content-type", "application/json").body(
                r#"{"clock":{"date":"2024-01-01","description":"Market is open","state":"open","timestamp":1,"next_change":"","next_state":"closed"}}"#,
            );
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_clock(None);
            assert!(resp.is_ok());
            op.assert();
        });
    }

    // 11. GET /v1/markets/calendar --------------------------------------------

    #[test]
    fn test_get_calendar_happy_path_verifies_month_year_params() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<GetCalendarResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/calendar")
                        .query_param("month", "6")
                        .query_param("year", "2024");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_calendar(
                        Some(CalendarMonth::new(6).unwrap()),
                        Some(CalendarYear::new(2024)),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    // 12. GET /v1/markets/search ----------------------------------------------

    #[test]
    fn test_search_companies_happy_path_returns_decoded_struct() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<SearchCompaniesResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/search")
                        .query_param("q", "apple")
                        .query_param("indexes", "false");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.search_companies("apple", Some(IndexesFlag::new(false)));
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    // 13. GET /v1/markets/lookup ----------------------------------------------

    #[test]
    fn test_lookup_symbol_happy_path_verifies_query_params() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(response in any::<LookupSymbolResponseWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.path("/v1/markets/lookup")
                        .query_param("q", "goog")
                        .query_param("exchanges", "N,Q")
                        .query_param("types", "stock,etf");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&response).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let exchanges = Exchanges::new(vec!["N".into(), "Q".into()]);
                    let types = SecurityTypes::new(vec!["stock".into(), "etf".into()]);
                    let resp = client.lookup_symbol(
                        "goog",
                        Some(&exchanges),
                        Some(&types),
                    );
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_lookup_symbol_with_empty_filters_omits_query_params() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/v1/markets/lookup")
                .query_param("q", "aap")
                .is_true(|req| {
                    req.query_params()
                        .iter()
                        .all(|(k, _)| k != "exchanges" && k != "types")
                });
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"securities": null}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.lookup_symbol("aap", None, None);
            assert!(resp.is_ok());
            op.assert();
        });
    }

    // Network error case — the server has no route for this path.
    #[test]
    fn test_network_error_when_server_refuses_connection() {
        // Point at a port no one is listening on.
        let mut cfg = Config::new();
        cfg.credentials.access_token = Some("t".into());
        cfg.rest_api.base_url = "http://127.0.0.1:1".into();
        let client = BlockingTradierRestClient::new(cfg).expect("client");
        let resp = client.get_etb_securities();
        assert!(resp.is_err());
    }
}

#[cfg(test)]
mod fundamentals_tests {
    use super::BlockingTradierRestClient;
    use crate::{
        common::Symbol,
        fundamentals::{
            api::blocking::Fundamentals,
            test_support::{
                CompanyResponseArrayWire, CorporateActionResponseArrayWire,
                CorporateCalendarResponseArrayWire, DividendResponseArrayWire,
                FinancialsResponseArrayWire, RatiosResponseArrayWire, StatisticsResponseArrayWire,
            },
        },
        utils::tests::with_env_vars,
        Config,
    };
    use httpmock::MockServer;
    use proptest::prelude::*;
    use std::cell::RefCell;

    fn make_symbol(s: &str) -> Symbol {
        s.parse().expect("valid symbol")
    }

    fn make_client(server: &MockServer) -> BlockingTradierRestClient {
        let mut cfg = Config::new();
        cfg.rest_api.base_url = server.base_url();
        BlockingTradierRestClient::new(cfg).expect("client to initialize")
    }

    fn run_with_env<F: FnOnce()>(server: &MockServer, f: F) {
        with_env_vars(
            vec![
                ("TRADIER_REST_BASE_URL", &server.base_url()),
                ("TRADIER_ACCESS_TOKEN", "testToken"),
            ],
            f,
        );
    }

    // -------- get_company -----------------------------------------------

    #[test]
    fn test_get_company_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<CompanyResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/company")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL,MSFT");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_company(&[make_symbol("AAPL"), make_symbol("MSFT")]);
                    op.assert();
                    assert!(resp.is_ok(), "resp: {:?}", resp.err());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_company_bad_request_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/company");
            then.status(400)
                .header("content-type", "application/json")
                .body(r#"{"fault":{"faultstring":"bad"}}"#);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_company(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_company_server_error_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/company");
            then.status(500);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_company(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_company_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/company");
            then.status(200)
                .header("content-type", "application/json")
                .body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_company(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // -------- get_corporate_calendars -----------------------------------

    #[test]
    fn test_get_corporate_calendars_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<CorporateCalendarResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/corporate_calendars")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_corporate_calendars(&[make_symbol("AAPL")]);
                    op.assert();
                    assert!(resp.is_ok(), "resp: {:?}", resp.err());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_corporate_calendars_server_error_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/corporate_calendars");
            then.status(503);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_corporate_calendars(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_corporate_calendars_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/corporate_calendars");
            then.status(200)
                .header("content-type", "application/json")
                .body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_corporate_calendars(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // -------- get_dividends ---------------------------------------------

    #[test]
    fn test_get_dividends_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<DividendResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/dividends")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL,MSFT");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client
                        .get_dividends(&[make_symbol("AAPL"), make_symbol("MSFT")]);
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_dividends_unauthorized_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/dividends");
            then.status(401);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_dividends(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_dividends_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/dividends");
            then.status(200).body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_dividends(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // -------- get_corporate_actions -------------------------------------

    #[test]
    fn test_get_corporate_actions_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<CorporateActionResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/corporate_actions")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_corporate_actions(&[make_symbol("AAPL")]);
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_corporate_actions_bad_request_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/corporate_actions");
            then.status(400);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_corporate_actions(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_corporate_actions_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/corporate_actions");
            then.status(200).body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_corporate_actions(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // -------- get_ratios ------------------------------------------------

    #[test]
    fn test_get_ratios_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<RatiosResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/ratios")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_ratios(&[make_symbol("AAPL")]);
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_ratios_server_error_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/ratios");
            then.status(500);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_ratios(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_ratios_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/ratios");
            then.status(200).body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_ratios(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // -------- get_financials --------------------------------------------

    #[test]
    fn test_get_financials_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<FinancialsResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/financials")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_financials(&[make_symbol("AAPL")]);
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_financials_server_error_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/financials");
            then.status(502);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_financials(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_financials_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/financials");
            then.status(200).body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_financials(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    // -------- get_statistics --------------------------------------------

    #[test]
    fn test_get_statistics_happy_path_verifies_query_and_header() {
        let server = RefCell::new(MockServer::start());
        proptest!(
            ProptestConfig::with_cases(8),
            |(wire in any::<StatisticsResponseArrayWire>())| {
                let server = server.borrow_mut();
                let mut op = server.mock(|when, then| {
                    when.method(httpmock::Method::GET)
                        .path("/beta/markets/fundamentals/statistics")
                        .header("accept", "application/json")
                        .query_param("symbols", "AAPL,MSFT,SPY");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body(serde_json::to_vec(&wire).expect("serialize"));
                });
                run_with_env(&server, || {
                    let client = make_client(&server);
                    let resp = client.get_statistics(&[
                        make_symbol("AAPL"),
                        make_symbol("MSFT"),
                        make_symbol("SPY"),
                    ]);
                    op.assert();
                    assert!(resp.is_ok());
                });
                op.delete();
            }
        );
    }

    #[test]
    fn test_get_statistics_server_error_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/statistics");
            then.status(500);
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_statistics(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_get_statistics_malformed_body_returns_error() {
        let server = MockServer::start();
        let op = server.mock(|when, then| {
            when.path("/beta/markets/fundamentals/statistics");
            then.status(200).body("{ not-json");
        });
        run_with_env(&server, || {
            let client = make_client(&server);
            let resp = client.get_statistics(&[make_symbol("AAPL")]);
            assert!(resp.is_err());
            op.assert();
        });
    }

    #[test]
    fn test_network_error_when_server_refuses_connection_fundamentals() {
        let mut cfg = Config::new();
        cfg.credentials.access_token = Some("t".into());
        cfg.rest_api.base_url = "http://127.0.0.1:1".into();
        let client = BlockingTradierRestClient::new(cfg).expect("client");
        let resp = client.get_company(&[make_symbol("AAPL")]);
        assert!(resp.is_err());
    }
}
