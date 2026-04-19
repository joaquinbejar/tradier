//! HTTP chunked-transfer streaming of Tradier events.
//!
//! Tradier exposes HTTP-streaming endpoints at
//! `https://stream.tradier.com/v1/markets/events`. The body is a
//! chunked stream of newline-delimited JSON events. Each line is one
//! event.
//!
//! This module provides [`market_events`] — a `Stream`-returning
//! helper that reuses the pooled `reqwest::Client` from
//! [`crate::client::non_blocking::TradierRestClient`]. Callers that
//! cannot use WebSockets (for instance, from behind strict corporate
//! egress filters) can fall back to this.
//!
//! ## Error handling
//!
//! - Non-2xx HTTP status is surfaced as
//!   [`crate::Error::NetworkError`] before the stream is constructed,
//!   so callers get the status synchronously from the returned
//!   `Result`.
//! - Per-chunk transport errors surface as `Err(Error::NetworkError)`
//!   items of the stream.
//! - Per-line decode failures surface as
//!   `Err(Error::StreamDecodeError(_, _))` items — consistent with the
//!   WebSocket decoder. Decode failures do NOT abort the stream.

use std::collections::VecDeque;

use futures_util::stream::{Stream, StreamExt};
use serde::Serialize;
use tracing::{debug, info, warn};

use crate::client::non_blocking::TradierRestClient;
use crate::wssession::events::MarketEvent;
use crate::{Error, Result};

/// Streams market events over the Tradier HTTP chunked-transfer
/// endpoint. The endpoint path is resolved against
/// [`crate::config::StreamingConfig::http_base_url`] with
/// `/v1/markets/events`.
///
/// # Parameters
///
/// - `client`: a [`TradierRestClient`] — the existing `reqwest::Client`
///   on it is reused.
/// - `session_id`: the session id minted by the REST session-bootstrap
///   call. Passed as the `sessionid` query parameter per the upstream
///   contract.
/// - `symbols`: symbols to subscribe to. Tradier accepts a
///   comma-separated list in the `symbols` query parameter.
/// - `filters`: optional list of event-type filters, matching
///   [`crate::wssession::MarketSessionFilter`] values (`quote`,
///   `trade`, `summary`, `timesale`, `tradex`). Passed as `filter`.
/// - `linebreak`: optional `linebreak=true|false`.
/// - `valid_only`: optional `validOnly=true|false`.
/// - `advanced_details`: optional `advancedDetails=true|false`.
///
/// # Errors
///
/// - [`Error::UrlParsingError`] if the HTTP streaming base URL is
///   malformed.
/// - [`Error::NetworkError`] if the initial request fails (connect /
///   handshake) OR the server responds with a non-2xx status.
/// - [`Error::MissingAccessToken`] if the client config has no access
///   token.
///
/// Once the stream is established, per-chunk and per-line errors
/// surface as `Err(_)` items of the stream.
pub async fn market_events(
    client: &TradierRestClient,
    session_id: &str,
    symbols: &[String],
    filters: Option<&[crate::wssession::MarketSessionFilter]>,
    linebreak: Option<bool>,
    valid_only: Option<bool>,
    advanced_details: Option<bool>,
) -> Result<impl Stream<Item = Result<MarketEvent>>> {
    let config = client.http_client_config();
    let base = &config.streaming.http_base_url;
    let url = format!("{base}/v1/markets/events");
    let bearer = client.get_bearer_token()?;

    // Tradier's HTTP streaming accepts the same filters as the WS
    // subscription payload. We serialize them to their `as_ref()`
    // string representations.
    let mut query: Vec<(&str, String)> = Vec::with_capacity(7);
    query.push(("sessionid", session_id.to_owned()));
    query.push(("symbols", symbols.join(",")));
    if let Some(filters) = filters {
        for f in filters {
            query.push(("filter", f.as_ref().to_owned()));
        }
    }
    if let Some(lb) = linebreak {
        query.push(("linebreak", lb.to_string()));
    }
    if let Some(v) = valid_only {
        query.push(("validOnly", v.to_string()));
    }
    if let Some(a) = advanced_details {
        query.push(("advancedDetails", a.to_string()));
    }

    info!(url = %url, "opening HTTP market event stream");
    let response = client
        .http_client()
        .get(&url)
        .bearer_auth(bearer)
        .header("accept", "application/json")
        .query(&query)
        .send()
        .await
        .map_err(Error::NetworkError)?;

    let response = error_for_non_success(response).await?;
    debug!("HTTP market stream accepted, decoding body");

    Ok(ndjson_event_stream(
        response.bytes_stream(),
        MarketEvent::from_json,
    ))
}

/// Opaque subscription query — currently unused but reserved for
/// callers that want to supply a typed struct rather than bare
/// parameters. Kept here so that future work can wire a builder
/// without reshaping the module surface.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct HttpStreamQuery<'a> {
    pub sessionid: &'a str,
}

/// Rejects non-2xx responses up front so that callers can decide
/// whether to reconnect without having to drain a body first.
async fn error_for_non_success(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    warn!(status = %status, "HTTP stream rejected");
    // Convert the status into a `reqwest::Error` via `error_for_status`
    // so the caller gets the HTTP status code preserved.
    match response.error_for_status() {
        Ok(r) => Ok(r), // unreachable in practice; keeps the type right
        Err(e) => Err(Error::NetworkError(e)),
    }
}

/// Internal decoder state shared across [`futures_util::stream::unfold`]
/// iterations: a chunked reqwest byte stream plus a rolling buffer of
/// partially-received bytes and a queue of decoded events waiting to
/// surface.
struct NdjsonState<S, T, F> {
    body: S,
    buffer: Vec<u8>,
    pending: VecDeque<Result<T>>,
    decode: F,
    finished: bool,
}

/// Wraps the body byte stream from a `reqwest::Response` in a
/// newline-delimited JSON decoder that yields `Result<T>` items.
///
/// - Transport errors become `Err(Error::NetworkError)` and terminate
///   the stream.
/// - Decode errors become `Err(Error::StreamDecodeError)` and are
///   yielded without terminating the stream.
/// - End-of-body flushes any remaining non-empty line.
#[inline]
fn ndjson_event_stream<S, B, T, F>(body: S, decode: F) -> impl Stream<Item = Result<T>>
where
    S: Stream<Item = reqwest::Result<B>> + Unpin,
    B: AsRef<[u8]>,
    F: Fn(&str) -> Result<T>,
{
    let state = NdjsonState {
        body,
        buffer: Vec::with_capacity(4096),
        pending: VecDeque::new(),
        decode,
        finished: false,
    };

    futures_util::stream::unfold(state, |mut state| async move {
        loop {
            if let Some(item) = state.pending.pop_front() {
                return Some((item, state));
            }
            if state.finished {
                return None;
            }
            match state.body.next().await {
                Some(Ok(chunk)) => {
                    state.buffer.extend_from_slice(chunk.as_ref());
                    drain_complete_lines(&mut state.buffer, &mut state.pending, &state.decode);
                }
                Some(Err(e)) => {
                    warn!(error = %e, "HTTP stream transport error");
                    state.finished = true;
                    state.pending.push_back(Err(Error::NetworkError(e)));
                }
                None => {
                    // Flush the final line, if any.
                    if !state.buffer.is_empty() {
                        let tail = std::mem::take(&mut state.buffer);
                        match std::str::from_utf8(&tail) {
                            Ok(s) => {
                                let trimmed = s.trim();
                                if !trimmed.is_empty() {
                                    state.pending.push_back((state.decode)(trimmed));
                                }
                            }
                            Err(e) => {
                                state.pending.push_back(Err(Error::StreamDecodeError(
                                    String::from_utf8_lossy(&tail).into_owned(),
                                    e.to_string(),
                                )));
                            }
                        }
                    }
                    state.finished = true;
                }
            }
        }
    })
}

/// Pulls every complete `\n`-terminated line out of `buffer` and pushes
/// the decoded result onto `pending`. Any partial trailing line stays
/// in `buffer` for the next chunk.
fn drain_complete_lines<T, F>(buffer: &mut Vec<u8>, pending: &mut VecDeque<Result<T>>, decode: &F)
where
    F: Fn(&str) -> Result<T>,
{
    while let Some(pos) = memchr_newline(buffer) {
        // Split off the first line including the newline.
        let rest = buffer.split_off(pos + 1);
        let line_bytes = std::mem::replace(buffer, rest);
        // Drop the trailing \n (and a preceding \r if present).
        let line_len = line_bytes.len().saturating_sub(1);
        let mut line = &line_bytes[..line_len];
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }
        if line.is_empty() {
            continue;
        }
        match std::str::from_utf8(line) {
            Ok(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    continue;
                }
                pending.push_back(decode(trimmed));
            }
            Err(e) => {
                pending.push_back(Err(Error::StreamDecodeError(
                    String::from_utf8_lossy(line).into_owned(),
                    e.to_string(),
                )));
            }
        }
    }
}

#[inline]
fn memchr_newline(haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|b| *b == b'\n')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Credentials, RestApiConfig, StreamingConfig};
    use httpmock::prelude::*;

    fn test_config(server_url: &str, stream_url: &str) -> Config {
        Config {
            credentials: Credentials {
                client_id: "test".into(),
                client_secret: "test".into(),
                access_token: Some("token".into()),
                refresh_token: None,
            },
            rest_api: RestApiConfig {
                base_url: server_url.to_string(),
                timeout: 30,
            },
            streaming: StreamingConfig {
                http_base_url: stream_url.to_string(),
                ws_base_url: String::new(),
                events_path: String::new(),
                reconnect_interval: 5,
            },
        }
    }

    #[tokio::test]
    async fn test_http_market_stream_decodes_newline_delimited_events() {
        let server = MockServer::start_async().await;
        let body = format!(
            "{quote}\n{trade}\n",
            quote = r#"{"type":"quote","symbol":"C","bid":281.84,"bidsz":60,"bidexch":"M","biddate":"1","ask":281.85,"asksz":6,"askexch":"Z","askdate":"2"}"#,
            trade = r#"{"type":"trade","symbol":"SPY","exch":"Q","price":"281.12","size":"100","cvol":"1","date":"3","last":"281.12"}"#,
        );
        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/markets/events");
                then.status(200)
                    .header("content-type", "application/json")
                    .header("transfer-encoding", "chunked")
                    .body(body);
            })
            .await;

        let config = test_config(&server.base_url(), &server.base_url());
        let client = TradierRestClient::new(config);
        let symbols = ["SPY".to_string()];
        let stream = market_events(&client, "sid", &symbols, None, None, None, None)
            .await
            .expect("market_events");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 2);
        assert!(matches!(collected[0], Ok(MarketEvent::Quote(_))));
        assert!(matches!(collected[1], Ok(MarketEvent::Trade(_))));
    }

    #[tokio::test]
    async fn test_http_market_stream_malformed_line_yields_decode_error_and_continues() {
        let server = MockServer::start_async().await;
        let body = format!(
            "{quote}\nnot-json\n{trade}\n",
            quote = r#"{"type":"quote","symbol":"C","bid":281.84,"bidsz":60,"bidexch":"M","biddate":"1","ask":281.85,"asksz":6,"askexch":"Z","askdate":"2"}"#,
            trade = r#"{"type":"trade","symbol":"SPY","exch":"Q","price":"281.12","size":"100","cvol":"1","date":"3","last":"281.12"}"#,
        );
        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/markets/events");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(body);
            })
            .await;

        let config = test_config(&server.base_url(), &server.base_url());
        let client = TradierRestClient::new(config);
        let symbols = ["SPY".to_string()];
        let stream = market_events(&client, "sid", &symbols, None, None, None, None)
            .await
            .expect("market_events");
        let collected: Vec<Result<MarketEvent>> = stream.collect().await;
        assert_eq!(collected.len(), 3, "got {collected:?}");
        assert!(matches!(collected[0], Ok(MarketEvent::Quote(_))));
        assert!(matches!(collected[1], Err(Error::StreamDecodeError(_, _))));
        assert!(matches!(collected[2], Ok(MarketEvent::Trade(_))));
    }

    #[tokio::test]
    async fn test_http_market_stream_non_success_surfaces_network_error() {
        let server = MockServer::start_async().await;
        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/markets/events");
                then.status(401)
                    .header("content-type", "application/json")
                    .body("unauthorized");
            })
            .await;

        let config = test_config(&server.base_url(), &server.base_url());
        let client = TradierRestClient::new(config);
        let symbols = ["SPY".to_string()];
        let result = market_events(&client, "sid", &symbols, None, None, None, None).await;
        assert!(matches!(result, Err(Error::NetworkError(_))));
    }
}
