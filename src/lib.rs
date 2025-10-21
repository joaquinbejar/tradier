/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

pub mod config;
pub mod constants;
mod error;
pub mod http;
pub mod utils;
pub mod wssession;
pub use error::{Error, Result};
#[cfg(test)]
pub mod test_support;
