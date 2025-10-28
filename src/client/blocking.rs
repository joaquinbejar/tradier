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
use tokio::runtime::{Handle, Runtime};

use crate::{
    accounts::types::{AccountNumber, GetAccountBalancesResponse},
    accounts::{api::blocking::Accounts, api::non_blocking::Accounts as NonBlockingAccounts},
    client::non_blocking::TradierRestClient as AsyncClient,
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
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;

    use crate::{
        accounts::test_support::GetAccountBalancesResponseWire,
        user::test_support::GetUserProfileResponseWire, utils::tests::with_env_vars, Config,
    };

    use httpmock::MockServer;
    use proptest::prelude::*;

    #[test]
    fn test_get_user_profile() {
        let server = MockServer::start();
        let server = RefCell::new(server);

        proptest!(|(response in any::<GetUserProfileResponseWire>())| {
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
    }

    #[test]
    fn test_get_account_balances() {
        let server = MockServer::start();
        let server = RefCell::new(server);

        proptest!(|(response in any::<GetAccountBalancesResponseWire>(),
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
    }

    #[tokio::test]
    async fn test_should_not_be_able_to_create_within_an_async_runtime() {
        let config = Config::new();
        let sut = BlockingTradierRestClient::new(config);
        assert!(sut.is_err());
    }
}
