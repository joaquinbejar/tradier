use crate::{
    config::{Config, Credentials, RestApiConfig, StreamingConfig},
    wssession::MarketSessionPayload,
};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::{frame::coding::CloseCode, CloseFrame, Message};

#[macro_export]
macro_rules! assert_decimal_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left: Decimal = $left;
        let right: Decimal = $right;
        let epsilon: Decimal = $epsilon;

        let abs_diff = (left - right).abs();
        let max_abs = left.abs().max(right.abs());

        if max_abs == Decimal::ZERO {
            assert!(
                abs_diff <= epsilon,
                "assertion failed: `(left == right)` \
                 (left: `{}`, right: `{}`, expected diff: `{}`, real diff: `{}`)",
                left,
                right,
                epsilon,
                abs_diff
            );
        } else {
            let relative_diff = abs_diff / max_abs;
            assert!(
                relative_diff <= epsilon,
                "assertion failed: `(left ≈ right)` \
                 (left: `{}`, right: `{}`, expected relative diff: `{}`, real relative diff: `{}`)",
                left,
                right,
                epsilon,
                relative_diff
            );
        }
    }};
}

// Helper function to create a test config
#[bon::builder(finish_fn = finish)]
#[allow(dead_code)]
pub(crate) fn create_test_config(
    server_url: &str,
    #[builder(default)] web_socket_url: &str,
    #[builder(default)] web_socket_path: &str,
    #[builder(default)] is_sandbox: bool,
) -> Config {
    Config {
        credentials: Credentials {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            access_token: Some("test_token".to_string()),
            refresh_token: None,
        },
        rest_api: RestApiConfig {
            base_url: if is_sandbox {
                "https://sandbox.tradier.com".to_string()
            } else {
                server_url.to_string()
            },
            timeout: 30,
        },
        streaming: StreamingConfig {
            http_base_url: "".to_string(),
            ws_base_url: web_socket_url.to_string(),
            events_path: web_socket_path.to_string(),
            reconnect_interval: 5,
        },
    }
}

#[bon::builder(finish_fn = create)]
#[cfg(test)]
pub(crate) async fn mock_websocket_server(
    #[builder(with = |a: &'static str, p: u16| (a, p) )] address: (&str, u16),
    expected_request: MarketSessionPayload<'_>,
    expected_response: &'static str,
) {
    let expected_request = serde_json::to_string(&expected_request).expect("serialization to work");
    let server = TcpListener::bind(address).await.unwrap();
    tokio::spawn(async move {
        let (stream, _) = server.accept().await.unwrap();
        let mut websocket = accept_async(stream).await.unwrap();
        if let Some(Ok(Message::Text(msg))) = websocket.next().await {
            assert_eq!(msg, expected_request);
            websocket
                .send(Message::Text(expected_response.into()))
                .await
                .unwrap();
            websocket
                .close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: "All Done!".into(),
                }))
                .await
                .unwrap();
        } else {
            panic!("Shouldn't be here");
        }
    });
}
