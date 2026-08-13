//! Shared error type -> HTTP response mapping (task 3.6).
//!
//! Every handler returns `Result<T, AppError>`. Validation errors render a
//! small Datastar-friendly fragment (a `#form-error` element) so a form
//! submission can patch its own error slot; everything else maps to a bare
//! status code, since those paths aren't meant to be shown to a visitor as
//! prose.

use askama::Template;
use axum::{
    http::StatusCode,
    response::{
        sse::Sse,
        IntoResponse, Response,
    },
};
use futures_util::stream;
use std::convert::Infallible;

use crate::templates::render;

#[derive(Template)]
#[template(path = "partials/form_error.html")]
struct FormErrorTemplate<'a> {
    id: &'a str,
    message: &'a str,
}

/// The `id` a bare `AppError::Validation(...)` targets — matches every
/// page's error slot except the account page, which has two forms (and
/// so two slots) on screen at once; see `ValidationAt`.
const DEFAULT_FORM_ERROR_ID: &str = "form-error";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A user-facing validation problem (bad form input, business rule
    /// violation). Rendered as a fragment so Datastar can patch it into
    /// the offending form's `#form-error` slot.
    #[error("{0}")]
    Validation(String),

    /// As `Validation`, but for a page with more than one form/error
    /// slot on screen simultaneously — targets `#{1}` instead of the
    /// default `#form-error`. Only the account page's password-change
    /// form needs this today (its own `#password-form-error`, distinct
    /// from the profile form's `#form-error` right above it on the same
    /// page) — a plain `Validation` there would morph the *wrong* slot:
    /// `#form-error` does exist on that page (it's the profile form's),
    /// so the message would silently land next to the wrong form
    /// instead of simply failing to appear.
    #[error("{0}")]
    ValidationAt(String, &'static str),

    /// No session / bad credentials on a route that requires one.
    #[error("unauthorized")]
    Unauthorized,

    /// Authenticated, but not allowed to do this. Not currently
    /// constructed anywhere: every ownership check in this codebase
    /// (rating deletion, image owner-preview) deliberately answers with
    /// `NotFound` instead, to avoid confirming a pending/unapproved
    /// row's existence to someone who isn't its owner. Kept as a
    /// distinct variant from `Unauthorized` for whichever future route
    /// *does* want to reveal "this exists, you just can't touch it".
    #[allow(dead_code)]
    #[error("forbidden")]
    Forbidden,

    /// Row doesn't exist, or exists but is filtered out (unapproved /
    /// soft-deleted) for the requester.
    #[error("not found")]
    NotFound,

    /// Row exists but is in the wrong state for the requested action
    /// (e.g. editing an already-deleted row, deleting an already-deleted
    /// row).
    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Validation(message) => {
                // Must be a genuine `text/event-stream` 200, exactly like
                // every success-path handler — confirmed by reading the
                // vendored datastar.js bundle's fetch handling: it checks
                // `response.status !== 200` and returns *before* even
                // looking at content-type, so the previous
                // `(422, body).into_response()` (a bare `text/plain`
                // body, since axum's `impl IntoResponse for String`
                // defaults to that content-type) was silently discarded
                // client-side — every validation error in the app was
                // failing with zero visible feedback, caught only by
                // driving a real browser (curl can't see this: the HTTP
                // response itself was "correct", nothing in it signals
                // that the browser will never apply it).
                let body = render(FormErrorTemplate { id: DEFAULT_FORM_ERROR_ID, message: &message });
                let event = crate::sse::patch_elements(&body);
                Sse::new(stream::iter(vec![Ok::<_, Infallible>(event)])).into_response()
            }
            AppError::ValidationAt(message, target_id) => {
                let body = render(FormErrorTemplate { id: target_id, message: &message });
                let event = crate::sse::patch_elements(&body);
                Sse::new(stream::iter(vec![Ok::<_, Infallible>(event)])).into_response()
            }
            AppError::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            AppError::Forbidden => StatusCode::FORBIDDEN.into_response(),
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message).into_response(),
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::Other(err) => {
                tracing::error!(error = %err, "internal error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
