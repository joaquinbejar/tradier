/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

mod config;
mod constants;
mod error;
pub mod utils;
pub mod wssession;
pub use error::{Error, Result};

mod accounts;
mod client;
pub mod common;
mod fundamentals;
mod market_data;
pub mod streaming;
mod trading;
mod user;
mod watchlists;

pub mod types {
    pub use crate::accounts::types::*;
    pub use crate::common::SortOrder;
    pub use crate::fundamentals::types::{
        AssetClassification, CashDividend, CompanyProfile, CompanyResponse, CompanyResult,
        CompanyTables, CorporateActionResponse, CorporateActionResult, CorporateActionTables,
        CorporateCalendarEvent, CorporateCalendarResponse, CorporateCalendarResult,
        CorporateCalendarTables, DividendResponse, DividendResult, DividendTables, EarningRatios,
        FinancialStatement, FinancialsResponse, FinancialsResult, FinancialsTables,
        FundamentalsEnvelope, Headquarter, MergerAcquisition, OperationRatios, PriceStatistics,
        RatiosResponse, RatiosResult, RatiosTables, ShareClass, ShareClassProfile,
        StatisticsResponse, StatisticsResult, StatisticsTables, StockSplit, TrailingReturns,
        ValuationRatios,
    };
    pub use crate::market_data::types::*;
    pub use crate::user::types::*;
    pub use crate::utils::OneOrMany;
}
pub mod blocking {
    pub use super::client::blocking::BlockingTradierRestClient as Client;
    pub mod operation {
        pub use crate::accounts::api::blocking::Accounts;
        pub use crate::fundamentals::api::blocking::Fundamentals;
        pub use crate::market_data::api::blocking::MarketData;
        pub use crate::user::api::blocking::User;
    }
}

pub mod non_blocking {
    pub use super::client::non_blocking::TradierRestClient as Client;
    pub mod operation {
        pub use crate::accounts::api::non_blocking::Accounts;
        pub use crate::fundamentals::api::non_blocking::Fundamentals;
        pub use crate::market_data::api::non_blocking::MarketData;
        pub use crate::user::api::non_blocking::User;
    }
}

pub use config::Config;
