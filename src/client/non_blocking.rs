use chrono::NaiveDate;
use url::Url;

use crate::{
    accounts::{
        api::non_blocking::Accounts,
        types::{AccountNumber, EventType, GetAccountBalancesResponse, Limit, Page},
    },
    config::Config,
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
        page: Option<&Page>,
        limit: Option<&Limit>,
        event_types: Option<&[EventType]>,
        start: Option<&NaiveDate>,
        end: Option<&NaiveDate>,
        symbol: Option<&str>,
        exact_match: Option<bool>,
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
            if let Some(event_types) = event_types {
                if !event_types.is_empty() {
                    let joined = event_types
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    query_pairs.append_pair("type", &joined);
                }
            }
            if let Some(start) = start {
                query_pairs.append_pair("start", &start.to_string());
            }
            if let Some(end) = end {
                query_pairs.append_pair("end", &end.to_string());
            }
            if let Some(symbol) = symbol {
                query_pairs.append_pair("symbol", symbol);
            }
            if let Some(exact_match) = exact_match {
                query_pairs.append_pair("exactMatch", if exact_match { "true" } else { "false" });
            }
        }
        let bearer_auth = self.get_bearer_token()?;
        let raw_response = self.make_service_call(url, bearer_auth).await?;
        raw_response
            .json::<GetAccountHistoryResponse>()
            .await
            .map_err(Error::NetworkError)
    }
}
