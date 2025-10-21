use url::Url;

use super::user::UserProfileResponse;
use crate::config::Config;

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
}

pub mod blocking {
    use tokio::runtime::Runtime;

    use super::*;

    pub struct TradierRestClient {
        rest_client: super::TradierRestClient,
        runtime: Runtime,
    }

    impl TradierRestClient {
        pub fn new(config: Config) -> crate::error::Result<Self> {
            Ok(Self {
                rest_client: super::TradierRestClient::new(config),
                runtime: tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?,
            })
        }
        pub fn get_user_profile(&self) -> crate::error::Result<UserProfileResponse> {
            self.runtime.block_on(self.rest_client.get_user_profile())
        }
    }
}

impl TradierRestClient {
    pub async fn get_user_profile(&self) -> crate::error::Result<UserProfileResponse> {
        let url = Url::parse(&self.http_client_config.rest_api.base_url)?;
        let url = url.join("/v1/user/profile")?;
        let bearer_auth = self
            .http_client_config
            .credentials
            .access_token
            .clone()
            .ok_or(crate::Error::MissingAccessToken)?;
        let raw_response = self
            .http_client
            .get(url)
            .bearer_auth(bearer_auth)
            .header("accept", "application/json")
            .send()
            .await?;
        raw_response
            .json::<UserProfileResponse>()
            .await
            .map_err(crate::Error::NetworkError)
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::{config::Config, test_support::with_env_vars, utils::logger::setup_logger};

    use super::blocking::TradierRestClient;
    use crate::test_support::GetUserProfileResponseWire;
    use httpmock::MockServer;
    use proptest::prelude::*;

    #[test]
    fn test_http_client() {
        setup_logger();
        let server = MockServer::start();
        let server = RefCell::new(server);

        proptest!(|(response in any::<GetUserProfileResponseWire>())| {
            let server = server.borrow_mut();
            let mut operation = server.mock(|when, then| {
                when.path("/v1/user/profile")
                    .header("accept", "application/json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&response)
                        .expect("serialization of wire type for tests to work"));
            });

            with_env_vars(vec![("TRADIER_REST_BASE_URL", &server.base_url()),
            ("TRADIER_ACCESS_TOKEN", "testToken")], || {
                let config = Config::new();
                let sut = TradierRestClient::new(config).expect("client to initialize");
                let response = dbg!(sut.get_user_profile());
                operation.assert();
                assert_eq!(operation.calls(), 1);
                assert!(response.is_ok());
                operation.delete();
            });
        });
    }
}
