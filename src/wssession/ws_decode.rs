//! Shared newline-delimited JSON decoder over a WebSocket read half.
//!
//! Both [`crate::wssession::MarketSession`] and
//! [`crate::wssession::AccountSession`] use this plumbing to turn a
//! `Stream<Item = tungstenite::Result<Message>>` into a
//! `Stream<Item = Result<T>>` where `T` is a typed event.
//!
//! The decoder:
//! - Splits `Message::Text` (and `Message::Binary`, after lossy UTF-8
//!   conversion) frames on `\n` and feeds each non-empty line through
//!   the caller-provided `decode` function.
//! - Yields decode failures as `Err(Error::StreamDecodeError(_, _))`
//!   items; it does NOT abort the stream on a decode failure.
//! - Treats `Message::Close` as end-of-stream (no error yielded).
//! - Treats transport errors as `Err(Error::WebSocketError(_))` items
//!   and then ends the stream.
//! - Ignores `Ping`, `Pong`, and raw `Frame` messages;
//!   `tokio_tungstenite` handles the automatic `Pong` reply.

use futures_util::stream::{self, Stream, StreamExt};
use std::collections::VecDeque;
use tracing::warn;
use tungstenite::Message;

use crate::Result;

/// State carried across calls to [`stream::unfold`].
struct DecoderState<S, T, F> {
    read: S,
    pending: VecDeque<Result<T>>,
    decode: F,
    finished: bool,
}

/// Drives a WebSocket read half and yields decoded events.
///
/// `decode` is applied to every non-empty, newline-separated JSON line
/// from a `Message::Text` (or `Message::Binary`) frame.
#[inline]
pub(super) fn ws_event_stream<S, T, F>(read: S, decode: F) -> impl Stream<Item = Result<T>>
where
    S: Stream<Item = std::result::Result<Message, tungstenite::Error>> + Unpin,
    F: Fn(&str) -> Result<T>,
{
    let state = DecoderState {
        read,
        pending: VecDeque::new(),
        decode,
        finished: false,
    };

    stream::unfold(state, |mut state| async move {
        loop {
            if let Some(item) = state.pending.pop_front() {
                return Some((item, state));
            }
            if state.finished {
                return None;
            }
            match state.read.next().await {
                Some(Ok(Message::Text(text))) => {
                    push_lines(&mut state.pending, text.as_ref(), &state.decode);
                }
                Some(Ok(Message::Binary(bytes))) => {
                    let text = String::from_utf8_lossy(bytes.as_ref());
                    push_lines(&mut state.pending, text.as_ref(), &state.decode);
                }
                Some(Ok(Message::Close(_))) => {
                    state.finished = true;
                }
                Some(Ok(Message::Ping(_)))
                | Some(Ok(Message::Pong(_)))
                | Some(Ok(Message::Frame(_))) => {
                    // tokio-tungstenite answers pings automatically; no work.
                }
                Some(Err(e)) => {
                    warn!(error = %e, "websocket transport error, terminating stream");
                    state.finished = true;
                    state
                        .pending
                        .push_back(Err(crate::Error::WebSocketError(Box::new(e))));
                }
                None => {
                    state.finished = true;
                }
            }
        }
    })
}

#[inline]
fn push_lines<T, F>(out: &mut VecDeque<Result<T>>, text: &str, decode: &F)
where
    F: Fn(&str) -> Result<T>,
{
    for line in text.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push_back(decode(trimmed));
    }
}
