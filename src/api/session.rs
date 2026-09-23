//! Who is visiting, and everything about their own account: login,
//! registration, logout, profile and password.

use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::{AccountData, SessionInfo, SessionUser};

#[cfg(feature = "server")]
use crate::{
    api::error::AppError,
    credentials,
    server::{auth, db, pool, routes::locale_from_headers},
};

/// The visitor and their language. Awaited by the app root during server
/// rendering, so the HTML and the hydrating client agree on both.
#[get("/api/session", session: tower_sessions::Session, headers: dioxus::fullstack::HeaderMap)]
pub async fn session_info() -> ApiResult<SessionInfo> {
    let user = auth::current_user(&session).await?;
    Ok(SessionInfo { user: user.map(|u| u.to_session()), locale: locale_from_headers(&headers) })
}

#[post("/api/login", session: tower_sessions::Session)]
pub async fn login(email: String, password: String) -> ApiResult<SessionUser> {
    let email = email.trim().to_lowercase();
    let Some(user) = db::user::find_by_email(pool(), &email).await? else {
        // The identifier, never the password: a run of these against one
        // address is what credential stuffing looks like.
        tracing::warn!(email = %email, "login failed: unknown email");
        return Err(AppError::invalid("error-login-failed"));
    };
    if !auth::verify_password(&password, &user.pwd_hash) {
        tracing::warn!(user_id = %user.id, "login failed: wrong password");
        return Err(AppError::invalid("error-login-failed"));
    }
    auth::log_in(&session, user.id).await?;
    tracing::info!(user_id = %user.id, "user logged in");
    Ok(user.to_session())
}

#[post("/api/register", session: tower_sessions::Session)]
pub async fn register(name: String, email: String, password: String) -> ApiResult<SessionUser> {
    let name = name.trim();
    let email = email.trim().to_lowercase();
    if name.is_empty() {
        return Err(AppError::invalid("error-name-required"));
    }
    if !credentials::valid_email(&email) {
        return Err(AppError::invalid("error-email-invalid"));
    }
    // The checklist in the form is a hint; this is the rule.
    if let Err(rule) = credentials::check_password(&password, name, &email) {
        return Err(AppError::invalid(rule.error_key()));
    }
    if db::user::email_exists(pool(), &email).await? {
        return Err(AppError::invalid("error-email-taken"));
    }
    let hash = auth::hash_password(&password)?;
    let user = db::user::insert(pool(), name, &email, &hash).await?;
    auth::log_in(&session, user.id).await?;
    tracing::info!(user_id = %user.id, "user registered");
    Ok(user.to_session())
}

#[post("/api/logout", session: tower_sessions::Session)]
pub async fn logout() -> ApiResult<()> {
    auth::log_out(&session).await?;
    Ok(())
}

/// The account page: profile plus the viewer's own pending submissions.
#[get("/api/account", session: tower_sessions::Session)]
pub async fn account_data() -> ApiResult<AccountData> {
    let user = auth::require_user(&session).await?;
    Ok(AccountData {
        pending: db::pending::for_user(pool(), user.id).await?,
        name: user.name,
        email: user.email,
    })
}

#[post("/api/account", session: tower_sessions::Session)]
pub async fn update_profile(name: String, email: String) -> ApiResult<SessionUser> {
    let user = auth::require_user(&session).await?;
    let name = name.trim();
    let email = email.trim().to_lowercase();
    if name.is_empty() {
        return Err(AppError::invalid("error-name-required"));
    }
    if !credentials::valid_email(&email) {
        return Err(AppError::invalid("error-email-invalid"));
    }
    if db::user::email_taken_by_other(pool(), &email, user.id).await? {
        return Err(AppError::invalid("error-email-taken"));
    }
    let updated = db::user::update_profile(pool(), user.id, name, &email).await?;
    tracing::info!(user_id = %user.id, "profile updated");
    Ok(updated.to_session())
}

#[post("/api/account/password", session: tower_sessions::Session)]
pub async fn change_password(current_password: String, new_password: String) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    if !auth::verify_password(&current_password, &user.pwd_hash) {
        tracing::warn!(user_id = %user.id, "password change rejected: wrong current password");
        return Err(AppError::invalid("error-current-password-wrong"));
    }
    // Checked against the stored name/email — the profile form saves
    // separately.
    if let Err(rule) = credentials::check_password(&new_password, &user.name, &user.email) {
        return Err(AppError::invalid(rule.error_key()));
    }
    let hash = auth::hash_password(&new_password)?;
    db::user::update_password(pool(), user.id, &hash).await?;
    tracing::info!(user_id = %user.id, "password changed");
    Ok(())
}
