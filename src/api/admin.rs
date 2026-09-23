//! The admin area: moderation queues (approve, reject, revert, restore),
//! accounts, and the Impressum's contents. Every function 404s for
//! anyone who isn't an admin (`auth::require_admin`).

use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::{AdminUserRow, QueuePage, RailCount, SiteInfo};

#[cfg(feature = "server")]
use crate::{
    api::error::AppError,
    credentials,
    models::{ChangeRow, Entity, QueueRow, QueueTab},
    server::{
        auth::{self, SEED_ADMIN_EMAIL},
        db::{self, moderation::Outcome},
        pool,
    },
};

#[cfg(feature = "server")]
fn human(at: time::OffsetDateTime) -> String {
    at.format(time::macros::format_description!("[day].[month].[year] [hour]:[minute]")).unwrap_or_default()
}

#[cfg(feature = "server")]
fn iso(at: time::OffsetDateTime) -> String {
    at.format(time::macros::format_description!("[year]-[month]-[day]")).unwrap_or_default()
}

#[cfg(feature = "server")]
fn entity(slug: &str) -> ApiResult<Entity> {
    Entity::from_slug(slug).ok_or_else(AppError::not_found)
}

/// The rail's pending badges, one per moderated table.
#[get("/api/admin/rail", session: tower_sessions::Session)]
pub async fn admin_rail() -> ApiResult<Vec<RailCount>> {
    auth::require_admin(&session).await?;
    let mut rail = Vec::new();
    for entity in Entity::ALL {
        rail.push(RailCount { entity, pending: db::moderation::counts(pool(), entity).await?.pending });
    }
    Ok(rail)
}

/// One queue tab. Only the visible tab's rows are loaded; the other two
/// are just counts.
#[get("/api/admin/queue/{slug}?tab", session: tower_sessions::Session)]
pub async fn admin_queue(slug: String, tab: String) -> ApiResult<QueuePage> {
    auth::require_admin(&session).await?;
    let entity = entity(&slug)?;
    let counts = db::moderation::counts(pool(), entity).await?;
    let to_row = |r: db::moderation::QueueRow| QueueRow {
        id: r.id,
        title: r.title,
        subtitle: r.subtitle,
        author: r.author,
        at_human: human(r.at),
        at_iso: iso(r.at),
    };
    let (rows, changes) = match QueueTab::from_key(&tab) {
        QueueTab::Pending => (db::moderation::pending(pool(), entity).await?.into_iter().map(to_row).collect(), vec![]),
        QueueTab::Deleted => (db::moderation::deleted(pool(), entity).await?.into_iter().map(to_row).collect(), vec![]),
        QueueTab::Changes => (
            vec![],
            db::moderation::changes(pool(), entity)
                .await?
                .into_iter()
                .map(|c| ChangeRow {
                    log_id: c.log_id,
                    title: c.title,
                    author: c.author,
                    at_human: human(c.at),
                    at_iso: iso(c.at),
                    diff: c.diff,
                })
                .collect(),
        ),
    };
    Ok(QueuePage { counts, rows, changes })
}

/// `action` is `approve`, `reject` or `restore`. Every one is reversible —
/// a rejection is a soft delete.
#[post("/api/admin/{slug}/{id}/{action}", session: tower_sessions::Session)]
pub async fn admin_moderate(slug: String, id: i64, action: String) -> ApiResult<()> {
    let admin = auth::require_admin(&session).await?;
    let entity = entity(&slug)?;
    let outcome = match action.as_str() {
        "approve" => db::moderation::approve(pool(), entity, id, admin.id).await?,
        "reject" => db::moderation::reject(pool(), entity, id, admin.id).await?,
        "restore" => db::moderation::restore(pool(), entity, id, admin.id).await?,
        _ => return Err(AppError::not_found()),
    };
    tracing::info!(entity = slug, id, action, "moderated");
    match outcome {
        Outcome::Done => Ok(()),
        Outcome::NameTaken => Err(AppError::invalid("admin-error-name-taken")),
    }
}

#[post("/api/admin/{slug}/revert/{log_id}", session: tower_sessions::Session)]
pub async fn admin_revert(slug: String, log_id: i64) -> ApiResult<()> {
    let admin = auth::require_admin(&session).await?;
    db::moderation::revert(pool(), entity(&slug)?, log_id, admin.id).await?;
    Ok(())
}

#[get("/api/admin/users", session: tower_sessions::Session)]
pub async fn admin_users() -> ApiResult<Vec<AdminUserRow>> {
    let admin = auth::require_admin(&session).await?;
    let admin_count = db::user::admin_count(pool()).await?;
    Ok(db::user::list_all(pool())
        .await?
        .into_iter()
        .map(|u| AdminUserRow {
            // The seed account is the documented way back in after a
            // lockout, so it keeps its rights and its row.
            protected: u.id == admin.id
                || (u.admin && admin_count <= 1)
                || u.email.eq_ignore_ascii_case(SEED_ADMIN_EMAIL),
            created_human: u.created.format(time::macros::format_description!("[day].[month].[year]")).unwrap_or_default(),
            id: u.id,
            name: u.name,
            email: u.email,
            admin: u.admin,
            contributions: u.contributions,
        })
        .collect())
}

/// An account created here is a real account: the same gates as public
/// registration apply.
#[post("/api/admin/users", session: tower_sessions::Session)]
pub async fn admin_create_user(name: String, email: String, password: String, admin: bool) -> ApiResult<()> {
    auth::require_admin(&session).await?;
    let name = name.trim();
    let email = email.trim().to_lowercase();
    if name.is_empty() || !credentials::valid_email(&email) {
        return Err(AppError::invalid("admin-users-error-invalid"));
    }
    if db::user::email_exists(pool(), &email).await? {
        return Err(AppError::invalid("admin-users-error-exists"));
    }
    if credentials::check_password(&password, name, &email).is_err() {
        return Err(AppError::invalid("admin-users-error-password"));
    }
    let hash = auth::hash_password(&password)?;
    let user = db::user::insert(pool(), name, &email, &hash).await?;
    if admin {
        db::user::set_admin(pool(), user.id, true).await?;
    }
    tracing::info!(user_id = user.id, "admin created user");
    Ok(())
}

/// Blocks the two ways to lock the door from inside: acting on your own
/// row, and removing the last admin. Re-checked here, not just hidden in
/// the UI — a request doesn't have to come from the button.
#[cfg(feature = "server")]
async fn guard_target(admin_id: i64, target_id: i64, make_admin: bool) -> ApiResult<()> {
    if target_id == admin_id {
        return Err(AppError::invalid("admin-users-error-self"));
    }
    let target = db::user::find_by_id(pool(), target_id).await?.ok_or_else(AppError::not_found)?;
    if target.email.eq_ignore_ascii_case(SEED_ADMIN_EMAIL) {
        return Err(AppError::invalid("admin-users-error-seed"));
    }
    if target.admin && !make_admin && db::user::admin_count(pool()).await? <= 1 {
        return Err(AppError::invalid("admin-users-error-last-admin"));
    }
    Ok(())
}

#[post("/api/admin/users/{id}/admin", session: tower_sessions::Session)]
pub async fn admin_set_admin(id: i64, admin: bool) -> ApiResult<()> {
    let me = auth::require_admin(&session).await?;
    guard_target(me.id, id, admin).await?;
    db::user::set_admin(pool(), id, admin).await?;
    tracing::info!(target_user = id, admin, "admin role changed");
    Ok(())
}

#[delete("/api/admin/users/{id}", session: tower_sessions::Session)]
pub async fn admin_delete_user(id: i64) -> ApiResult<()> {
    let me = auth::require_admin(&session).await?;
    guard_target(me.id, id, false).await?;
    db::user::delete(pool(), id).await?;
    tracing::info!(target_user = id, "admin deleted user");
    Ok(())
}

#[get("/api/admin/site-info", session: tower_sessions::Session)]
pub async fn admin_site_info() -> ApiResult<SiteInfo> {
    auth::require_admin(&session).await?;
    Ok(db::site_info::get(pool()).await?)
}

#[post("/api/admin/site-info", session: tower_sessions::Session)]
pub async fn admin_save_site_info(info: SiteInfo) -> ApiResult<()> {
    let admin = auth::require_admin(&session).await?;
    db::site_info::update(pool(), &info, admin.id).await?;
    tracing::info!(admin_id = admin.id, "site info updated");
    Ok(())
}
