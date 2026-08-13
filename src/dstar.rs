//! Extracting the current Datastar signal snapshot from a `@get`/`@delete`
//! request.
//!
//! Ground-truthed against the vendored `static/datastar.js` bundle (not
//! guessed): for POST/PUT/PATCH the signals are the JSON request body
//! (`Json<T>` handles that directly); for GET/DELETE, Datastar puts the
//! *entire* signals object, JSON-stringified, into a single query
//! parameter literally named `datastar` — not one query param per signal
//! key. `axum::extract::Query` alone can't parse that; this extractor
//! reads the `datastar` param and JSON-decodes it into `T`.

use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

use crate::error::AppError;

pub struct DatastarSignals<T>(pub T);

impl<S, T> FromRequestParts<S> for DatastarSignals<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Default,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Validation("invalid query string".into()))?;

        match params.get("datastar") {
            Some(raw) => {
                let value = serde_json::from_str(raw).map_err(|err| {
                    // A malformed `datastar` blob only happens from a
                    // hand-crafted or tampered request — the vendored
                    // client always sends valid JSON here — so it's worth
                    // a WARN rather than passing through silently as a
                    // routine validation failure.
                    tracing::warn!(error = %err, "malformed datastar signals payload");
                    AppError::Validation("invalid datastar payload".into())
                })?;
                Ok(DatastarSignals(value))
            }
            None => Ok(DatastarSignals(T::default())),
        }
    }
}
