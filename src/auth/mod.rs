pub mod password;

use axum::{extract::FromRequestParts, http::request::Parts};
use tower_sessions::Session;

use crate::{error::AppError, models::User, state::AppState};

const USER_ID_KEY: &str = "user_id";

pub async fn log_in(session: &Session, user_id: i64) -> anyhow::Result<()> {
    session.insert(USER_ID_KEY, user_id).await?;
    // Defends against session fixation: a fresh session id after a
    // privilege change (anonymous -> authenticated).
    session.cycle_id().await?;
    Ok(())
}

pub async fn log_out(session: &Session) -> anyhow::Result<()> {
    session.flush().await?;
    Ok(())
}

/// Task 3.3: required-auth extractor. Any route taking `CurrentUser`
/// returns 401 (via `AppError::Unauthorized`) when there's no valid
/// session — no ownership variant is layered on top of this, because
/// catalog-editing (§8) deliberately allows *any* authenticated user to
/// edit/delete *any* catalog entity. The routes that do need ownership
/// (rating deletion, account edits, image owner-preview) check
/// `created_by == user.id` themselves against the loaded row.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let user_id: Option<i64> = session.get(USER_ID_KEY).await.unwrap_or(None);
        let user_id = user_id.ok_or(AppError::Unauthorized)?;
        let user = crate::db::user::find_by_id(&state.pool, user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(CurrentUser(user))
    }
}

/// Read-optional variant for routes anonymous visitors can also hit (every
/// page/fragment handler needs to know `loggedIn` for the navbar).
pub struct OptionalUser(pub Option<User>);

impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match CurrentUser::from_request_parts(parts, state).await {
            Ok(CurrentUser(user)) => Ok(OptionalUser(Some(user))),
            Err(_) => Ok(OptionalUser(None)),
        }
    }
}
