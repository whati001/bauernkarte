//! Server functions — the app's whole HTTP API. Each is an ordinary async
//! fn the components call; on the client the macro turns the call into a
//! request, on the server it runs the body. They replace the original's
//! Datastar SSE handlers: those rendered HTML fragments server-side, these
//! return data and the components render it.
//!
//! Reads are `GET` (query-string arguments); everything that changes the
//! database is `POST`/`PATCH`/`DELETE`, which is what the rate limiter
//! keys on (`server::rate_limit`).

pub mod admin;
pub mod error;
pub mod image;
pub mod offer;
pub mod product;
pub mod search;
pub mod session;
pub mod site;
pub mod store;

/// Trimmed, or `None` when blank — for every optional text field.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn non_empty(s: &str) -> Option<&str> {
    Some(s.trim()).filter(|v| !v.is_empty())
}
