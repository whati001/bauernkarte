//! Sessions and passwords: Argon2id hashing, the session's user id, the
//! "who is asking" lookups every server function starts with, and the
//! one-time password for the seeded admin account.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;
use tower_sessions::Session;

use crate::{
    api::error::{ApiResult, AppError},
    credentials,
    server::db::user::{self, User},
};

const USER_ID_KEY: &str = "user_id";
pub const SEED_ADMIN_EMAIL: &str = "bauernkarte@rehka.dev";

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| anyhow::anyhow!("password hashing failed: {err}"))
}

pub fn verify_password(password: &str, encoded_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(encoded_hash) else {
        return false;
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

pub async fn log_in(session: &Session, user_id: i64) -> anyhow::Result<()> {
    session.insert(USER_ID_KEY, user_id).await?;
    // A fresh session id on privilege change defends against fixation.
    session.cycle_id().await?;
    Ok(())
}

pub async fn log_out(session: &Session) -> anyhow::Result<()> {
    session.flush().await?;
    Ok(())
}

/// The signed-in user, if any. A session pointing at a since-deleted
/// account counts as anonymous.
pub async fn current_user(session: &Session) -> ApiResult<Option<User>> {
    let user_id: Option<i64> = session.get(USER_ID_KEY).await.unwrap_or(None);
    match user_id {
        Some(id) => Ok(user::find_by_id(crate::server::pool(), id).await?),
        None => Ok(None),
    }
}

/// Catalog editing deliberately allows *any* signed-in user to edit any
/// entity, so there is no ownership variant of this.
pub async fn require_user(session: &Session) -> ApiResult<User> {
    current_user(session).await?.ok_or_else(AppError::unauthorized)
}

/// 404, not 403, for anyone who isn't an admin — a moderation URL that
/// answers "forbidden" confirms it exists.
pub async fn require_admin(session: &Session) -> ApiResult<User> {
    match current_user(session).await? {
        Some(user) if user.admin => Ok(user),
        _ => Err(AppError::not_found()),
    }
}

/// Gives the seeded `bauernkarte@rehka.dev` account its password on the
/// first startup that finds it without one. A migration can't: it has no
/// access to `.env` and a committed hash would be a published password.
/// Runs once — a password changed through the account page sticks.
pub async fn seed_admin_password(pool: &PgPool, admin_password: Option<&str>) -> anyhow::Result<()> {
    let needs_password = sqlx::query_scalar!(
        r#"select exists(select 1 from "user" where email = $1 and pwd_hash = '') as "exists!""#,
        SEED_ADMIN_EMAIL
    )
    .fetch_one(pool)
    .await?;
    if !needs_password {
        return Ok(());
    }
    let Some(password) = admin_password else {
        tracing::warn!(
            email = SEED_ADMIN_EMAIL,
            "admin account has no password and ADMIN_PASSWORD is unset — set it in .env and restart"
        );
        return Ok(());
    };
    // A weak ADMIN_PASSWORD fails loudly rather than quietly creating the
    // one account that can moderate everything.
    if let Err(rule) = credentials::check_password(password, "BauernKarte Admin", SEED_ADMIN_EMAIL) {
        anyhow::bail!("ADMIN_PASSWORD does not meet the password policy ({rule:?})");
    }
    let hash = hash_password(password)?;
    sqlx::query!(
        r#"update "user" set pwd_hash = $2, modified = now() where email = $1 and pwd_hash = ''"#,
        SEED_ADMIN_EMAIL,
        hash
    )
    .execute(pool)
    .await?;
    tracing::info!(email = SEED_ADMIN_EMAIL, "admin password set from ADMIN_PASSWORD");
    Ok(())
}
