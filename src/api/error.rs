//! The one error type every server function returns.
//!
//! It carries an i18n key rather than prose, so the message is shown in
//! the visitor's language no matter which side produced it (the client
//! translates with `Locale::t_error`). Internal failures are logged on
//! the server and reach the visitor only as `error-generic`.

use dioxus::fullstack::{AsStatusCode, ServerFnError, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppError {
    pub key: String,
    pub status: u16,
}

pub type ApiResult<T> = Result<T, AppError>;

impl AppError {
    /// Bad input or a broken business rule — shown next to the form.
    pub fn invalid(key: &str) -> Self {
        Self { key: key.to_string(), status: 422 }
    }

    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    pub fn unauthorized() -> Self {
        Self { key: "error-login-required".into(), status: 401 }
    }

    /// Also what a non-admin gets from an admin function: a 404 reveals
    /// nothing about whether the thing exists.
    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    pub fn not_found() -> Self {
        Self { key: "error-not-found".into(), status: 404 }
    }

    /// Acting on a row that has since been (soft-)deleted.
    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    pub fn deleted() -> Self {
        Self { key: "error-deleted".into(), status: 409 }
    }

    pub fn internal() -> Self {
        Self { key: "error-generic".into(), status: 500 }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key)
    }
}

impl AsStatusCode for AppError {
    fn as_status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Transport failures (network down, 429 from the rate limiter, …) have
/// no key of their own.
impl From<ServerFnError> for AppError {
    fn from(err: ServerFnError) -> Self {
        match err {
            ServerFnError::ServerError { message, code, .. } => Self { key: message, status: code },
            _ => Self::internal(),
        }
    }
}

#[cfg(feature = "server")]
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "database error");
        Self::internal()
    }
}

#[cfg(feature = "server")]
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(error = %err, "internal error");
        Self::internal()
    }
}
