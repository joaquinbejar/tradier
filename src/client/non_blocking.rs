use chrono::{DateTime, NaiveDate, Utc};
use url::Url;

use crate::{
    accounts::{
        api::non_blocking::Accounts,
        types::{
            AccountNumber, EventType, GainLossSortBy, GetAccountBalancesResponse,
            GetAccountGainLossResponse, GetAccountOrdersResponse, IncludeTags, Limit, Page,
        },
    },
    common::SortOrder,
    config::Config,
    market_data::{
        api::non_blocking::MarketData,
        types::{
            format_naive_date, format_timesales_datetime, CalendarMonth, CalendarYear, DelayedFlag,
            Exchanges, GetCalendarResponse, GetClockResponse, GetEtbSecuritiesResponse,
            GetHistoricalQuotesResponse, GetOptionChainsResponse, GetOptionExpirationsResponse,
            GetOptionStrikesResponse, GetQuotesResponse, GetTimeAndSalesResponse, Greeks,
            HistoryInterval, IncludeAllRoots, IncludeStrikes, IndexesFlag,
            LookupOptionSymbolsResponse, LookupSymbolResponse, SearchCompaniesResponse,
            SecurityTypes, SessionFilter, Symbol, Symbols, TimeSalesInterval,
        },
    },
    types::{GetAccountHistoryResponse, GetAccountPositionsResponse},
    user::{api::non_blocking::User, UserProfileResponse},
    utils::Sealed,
    Error, Result,
};

#[derive(Debug)]
pub struct TradierRestClient {
    http_client: reqwest::Client,
    http_client_config: Config,
}

impl TradierRestClient {
    pub fn new(config: Config) -> Self {
        TradierRestClient {
            http_client: reqwest::Client::new(),
            http_client_config: config,
        }
    }

    pub fn get_request_url(&self, url_path: &str) -> Result<Url> {
        Url::parse(&self.http_client_config.rest_api.base_url)?
            .join(url_path)
            .map_err(Error::UrlParsingError)
    }

    pub fn get_bearer_token(&self) -> Result<String> {
        self.http_client_config
            .credentials
            .access_token
            .clone()
            .ok_or(Error::MissingAccessToken)
    }

    pub async fn make_service_call(
        &self,
        url: Url,
        bearer_token: String,
    ) -> Result<reqwest::Response> {
        self.http_client
            .get(url)
            .bearer_auth(bearer_token)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(Error::NetworkError)
    }

    /// POSTs a form body and parses the JSON response into `T`.
    async fn post_form<T, I, K, V>(&self, url: Url, form: I) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let bearer = self.get_bearer_token()?;
        let pairs: Vec<(String, String)> = form
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()))
            .collect();
        self.http_client
            .post(url)
            .bearer_auth(bearer)
            .header("accept", "application/json")
            .form(&pairs)
            .send()
            .await
            .map_err(Error::NetworkError)?
            .json::<T>()
            .await
            .map_err(Error::NetworkError)
    }
}

impl Sealed for TradierRestClient {}

#[async_trait::async_trait]
impl User for TradierRestClient {
    async fn get_user_profile(&self) -> Result<UserProfileResponse> {
        let url = self.get_request_url("/v1/user/profile")?;
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<UserProfileResponse>()
            .await
            .map_err(Error::NetworkError)
    }
}

#[async_trait::async_trait]
impl Accounts for TradierRestClient {
    async fn get_account_balances(
        &self,
        account_id: &AccountNumber,
    ) -> Result<GetAccountBalancesResponse> {
        let url = self.get_request_url(&format!("/v1/accounts/{account_id}/balances"))?;
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<GetAccountBalancesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_account_positions(
        &self,
        account_id: &AccountNumber,
    ) -> Result<GetAccountPositionsResponse> {
        let url = self.get_request_url(&format!("/v1/accounts/{account_id}/positions"))?;
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<GetAccountPositionsResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_account_history(
        &self,
        account_id: &AccountNumber,
        page: Option<Page>,
        limit: Option<Limit>,
        event_type: Option<EventType>,
    ) -> Result<GetAccountHistoryResponse> {
        let mut url = self.get_request_url(&format!("/v1/accounts/{account_id}/history"))?;
        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(page) = page {
                query_pairs.append_pair("page", &page.to_string());
            }
            if let Some(limit) = limit {
                query_pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(event_type) = event_type {
                query_pairs.append_pair("type", &event_type.to_string());
            }
        }
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<GetAccountHistoryResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_account_gain_loss(
        &self,
        account_number: &AccountNumber,
        page: Option<Page>,
        limit: Option<Limit>,
        sort_by: Option<GainLossSortBy>,
        sort_order: Option<SortOrder>,
    ) -> Result<GetAccountGainLossResponse> {
        let mut url = self.get_request_url(&format!("/v1/accounts/{account_number}/gainloss"))?;
        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(page) = page {
                query_pairs.append_pair("page", &page.to_string());
            }
            if let Some(limit) = limit {
                query_pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(sort_by) = sort_by {
                query_pairs.append_pair("sortBy", &sort_by.to_string());
            }
            if let Some(sort_order) = sort_order {
                query_pairs.append_pair("sort", &sort_order.to_string());
            }
        }
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<GetAccountGainLossResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_account_orders(
        &self,
        account_id: &AccountNumber,
        page: &Page,
        limit: &Limit,
        include_tags: &IncludeTags,
    ) -> Result<GetAccountOrdersResponse> {
        let mut url = self.get_request_url(&format!("/v1/accounts/{account_id}/orders"))?;
        url.query_pairs_mut()
            .append_pair("page", &page.to_string())
            .append_pair("limit", &limit.to_string())
            .append_pair("includeTags", &include_tags.to_string());
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<GetAccountOrdersResponse>()
            .await
            .map_err(Error::NetworkError)
    }
}

#[async_trait::async_trait]
impl MarketData for TradierRestClient {
    async fn get_quotes(
        &self,
        symbols: &Symbols,
        greeks: Option<Greeks>,
    ) -> Result<GetQuotesResponse> {
        let mut url = self.get_request_url("/v1/markets/quotes")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("symbols", &symbols.to_string());
            if let Some(g) = greeks {
                qp.append_pair("greeks", &g.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetQuotesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn post_quotes(
        &self,
        symbols: &Symbols,
        greeks: Option<Greeks>,
    ) -> Result<GetQuotesResponse> {
        let url = self.get_request_url("/v1/markets/quotes")?;
        let mut form: Vec<(&str, String)> = Vec::with_capacity(2);
        form.push(("symbols", symbols.to_string()));
        if let Some(g) = greeks {
            form.push(("greeks", g.to_string()));
        }
        self.post_form::<GetQuotesResponse, _, _, _>(url, form)
            .await
    }

    async fn get_option_chains(
        &self,
        symbol: &Symbol,
        expiration: &NaiveDate,
        greeks: Option<Greeks>,
    ) -> Result<GetOptionChainsResponse> {
        let mut url = self.get_request_url("/v1/markets/options/chains")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("symbol", &symbol.to_string());
            qp.append_pair("expiration", &format_naive_date(expiration));
            if let Some(g) = greeks {
                qp.append_pair("greeks", &g.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetOptionChainsResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_option_strikes(
        &self,
        symbol: &Symbol,
        expiration: &NaiveDate,
    ) -> Result<GetOptionStrikesResponse> {
        let mut url = self.get_request_url("/v1/markets/options/strikes")?;
        url.query_pairs_mut()
            .append_pair("symbol", &symbol.to_string())
            .append_pair("expiration", &format_naive_date(expiration));
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetOptionStrikesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_option_expirations(
        &self,
        symbol: &Symbol,
        include_all_roots: Option<IncludeAllRoots>,
        strikes: Option<IncludeStrikes>,
    ) -> Result<GetOptionExpirationsResponse> {
        let mut url = self.get_request_url("/v1/markets/options/expirations")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("symbol", &symbol.to_string());
            if let Some(v) = include_all_roots {
                qp.append_pair("includeAllRoots", &v.to_string());
            }
            if let Some(v) = strikes {
                qp.append_pair("strikes", &v.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetOptionExpirationsResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn lookup_option_symbols(
        &self,
        underlying: &Symbol,
    ) -> Result<LookupOptionSymbolsResponse> {
        let mut url = self.get_request_url("/v1/markets/options/lookup")?;
        url.query_pairs_mut()
            .append_pair("underlying", &underlying.to_string());
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<LookupOptionSymbolsResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_historical_quotes(
        &self,
        symbol: &Symbol,
        interval: Option<HistoryInterval>,
        start: Option<&NaiveDate>,
        end: Option<&NaiveDate>,
        session_filter: Option<SessionFilter>,
    ) -> Result<GetHistoricalQuotesResponse> {
        let mut url = self.get_request_url("/v1/markets/history")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("symbol", &symbol.to_string());
            if let Some(i) = interval {
                qp.append_pair("interval", &i.to_string());
            }
            if let Some(s) = start {
                qp.append_pair("start", &format_naive_date(s));
            }
            if let Some(e) = end {
                qp.append_pair("end", &format_naive_date(e));
            }
            if let Some(f) = session_filter {
                qp.append_pair("session_filter", &f.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetHistoricalQuotesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_time_and_sales(
        &self,
        symbol: &Symbol,
        interval: Option<TimeSalesInterval>,
        start: Option<&DateTime<Utc>>,
        end: Option<&DateTime<Utc>>,
        session_filter: Option<SessionFilter>,
    ) -> Result<GetTimeAndSalesResponse> {
        let mut url = self.get_request_url("/v1/markets/timesales")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("symbol", &symbol.to_string());
            if let Some(i) = interval {
                qp.append_pair("interval", &i.to_string());
            }
            if let Some(s) = start {
                qp.append_pair("start", &format_timesales_datetime(s));
            }
            if let Some(e) = end {
                qp.append_pair("end", &format_timesales_datetime(e));
            }
            if let Some(f) = session_filter {
                qp.append_pair("session_filter", &f.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetTimeAndSalesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_etb_securities(&self) -> Result<GetEtbSecuritiesResponse> {
        let url = self.get_request_url("/v1/markets/etb")?;
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetEtbSecuritiesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_clock(&self, delayed: Option<DelayedFlag>) -> Result<GetClockResponse> {
        let mut url = self.get_request_url("/v1/markets/clock")?;
        if let Some(d) = delayed {
            url.query_pairs_mut().append_pair("delayed", &d.to_string());
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetClockResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn get_calendar(
        &self,
        month: Option<CalendarMonth>,
        year: Option<CalendarYear>,
    ) -> Result<GetCalendarResponse> {
        let mut url = self.get_request_url("/v1/markets/calendar")?;
        {
            let mut qp = url.query_pairs_mut();
            if let Some(m) = month {
                qp.append_pair("month", &m.to_string());
            }
            if let Some(y) = year {
                qp.append_pair("year", &y.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<GetCalendarResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn search_companies(
        &self,
        q: &str,
        indexes: Option<IndexesFlag>,
    ) -> Result<SearchCompaniesResponse> {
        let mut url = self.get_request_url("/v1/markets/search")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("q", q);
            if let Some(i) = indexes {
                qp.append_pair("indexes", &i.to_string());
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<SearchCompaniesResponse>()
            .await
            .map_err(Error::NetworkError)
    }

    async fn lookup_symbol(
        &self,
        q: &str,
        exchanges: Option<&Exchanges>,
        types: Option<&SecurityTypes>,
    ) -> Result<LookupSymbolResponse> {
        let mut url = self.get_request_url("/v1/markets/lookup")?;
        {
            let mut qp = url.query_pairs_mut();
            qp.append_pair("q", q);
            if let Some(ex) = exchanges {
                if !ex.is_empty() {
                    qp.append_pair("exchanges", &ex.to_string());
                }
            }
            if let Some(t) = types {
                if !t.is_empty() {
                    qp.append_pair("types", &t.to_string());
                }
            }
        }
        let bearer = self.get_bearer_token()?;
        self.make_service_call(url, bearer)
            .await?
            .json::<LookupSymbolResponse>()
            .await
            .map_err(Error::NetworkError)
    }
}
