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
mod market_data;
mod streaming;
mod trading;
mod user;
mod watchlists;

pub mod types {
    pub use crate::accounts::types::*;
    pub use crate::user::types::*;
    pub use crate::utils::OneOrMany;
}
pub mod blocking {
    pub use super::client::blocking::BlockingTradierRestClient as Client;
    pub mod operation {
        pub use crate::accounts::api::blocking::Accounts;
        pub use crate::user::api::blocking::User;
    }
}

pub mod non_blocking {
    pub use super::client::non_blocking::TradierRestClient as Client;
    pub mod operation {
        pub use crate::accounts::api::non_blocking::Accounts;
        pub use crate::user::api::non_blocking::User;
    }
}

pub use config::Config;
