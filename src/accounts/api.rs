use crate::accounts::types::{
    AccountNumber, EventType, GetAccountBalancesResponse, GetAccountHistoryResponse, Limit, Page,
};
use crate::types::GetAccountPositionsResponse;
use crate::{error::Result, utils::Sealed};

pub mod non_blocking {
    use super::*;

    #[async_trait::async_trait]
    pub trait Accounts: Sealed {
        async fn get_account_balances(
            &self,
            account_number: &AccountNumber,
        ) -> Result<GetAccountBalancesResponse>;

        async fn get_account_positions(
            &self,
            account_number: &AccountNumber,
        ) -> Result<GetAccountPositionsResponse>;

        async fn get_account_history(
            &self,
            account_number: &AccountNumber,
            page: Option<Page>,
            limit: Option<Limit>,
            event_type: Option<EventType>,
        ) -> Result<GetAccountHistoryResponse>;
    }
}
pub mod blocking {
    use super::*;

    pub trait Accounts: Sealed {
        fn get_account_balances(
            &self,
            account_number: &AccountNumber,
        ) -> Result<GetAccountBalancesResponse>;
        fn get_account_positions(
            &self,
            account_number: &AccountNumber,
        ) -> Result<GetAccountPositionsResponse>;
        fn get_account_history(
            &self,
            account_number: &AccountNumber,
            page: Option<Page>,
            limit: Option<Limit>,
            event_type: Option<EventType>,
        ) -> Result<GetAccountHistoryResponse>;
    }
}
