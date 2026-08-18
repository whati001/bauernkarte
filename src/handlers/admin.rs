//! The admin area: `/admin/*`. Replaces the hand-run SQL in
//! `RUNBOOK.md` with a UI — approve, reject, revert an edit, restore a
//! soft-deleted row, and manage accounts.
//!
//! Every route here takes `AdminUser`, which 404s for anyone else, so
//! there is no route in this file that a non-admin can reach.
//!
//! Unlike the map side this is plain full-page HTML with form POSTs and
//! redirects, not Datastar. The map is server-driven-with-SSE because a
//! search that reloaded the page would throw away the viewport; nothing
//! in here has that constraint, and POST-then-redirect gets working
//! back/forward, refresh-safety and no signal set for free.

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use time::{format_description::FormatItem, macros::format_description};

use crate::{
    auth::{admin_seed::SEED_ADMIN_EMAIL, password, AdminUser},
    db::{
        self,
        moderation::{Entity, Outcome},
    },
    error::{AppError, AppResult},
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    state::AppState,
    templates::render,
};

use askama::Template;

/// Dates in this area are only ever "when did this land in the queue",
/// so a plain day/time is enough — no locale-aware month names to
/// maintain, and it sorts the way it reads.
const HUMAN: &[FormatItem<'static>] = format_description!("[day].[month].[year] [hour]:[minute]");
const HUMAN_DAY: &[FormatItem<'static>] = format_description!("[day].[month].[year]");
const ISO: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

fn human(at: time::OffsetDateTime) -> String {
    at.format(HUMAN).unwrap_or_default()
}
fn human_day(at: time::OffsetDateTime) -> String {
    at.format(HUMAN_DAY).unwrap_or_default()
}
fn iso(at: time::OffsetDateTime) -> String {
    at.format(ISO).unwrap_or_default()
}

// ---------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------

struct RailSection {
    slug: &'static str,
    label: String,
    pending: i64,
}

#[derive(Template)]
#[template(path = "admin/layout.html")]
struct AdminLayout {
    title: String,
    current_locale: &'static str,
    navbar_html: String,
    section: String,
    sections: Vec<RailSection>,
    content_html: String,
    flash: Option<String>,
    flash_error: Option<String>,
}

/// Renders the shell around an already-rendered content fragment. The
/// rail's pending counts come from one `counts` call per entity — five
/// cheap aggregate queries, run on every admin page so the badges are
/// never stale relative to what the visitor is looking at.
async fn shell(
    state: &AppState,
    admin: &crate::models::User,
    title: &str,
    section: &str,
    content_html: String,
    flash: Option<String>,
    flash_error: Option<String>,
) -> AppResult<Response> {
    let locale = i18n::current_locale();
    let mut sections = Vec::with_capacity(Entity::ALL.len());
    for entity in Entity::ALL {
        let counts = db::moderation::counts(&state.pool, entity).await?;
        sections.push(RailSection {
            slug: entity.slug(),
            label: i18n::translate(locale, entity.label_key()),
            pending: counts.pending,
        });
    }

    let nav_products = db::product::list_top_rated(&state.pool, 0).await?;
    let navbar_html = crate::templates::render_navbar(Some(admin), nav_products);

    let page = AdminLayout {
        title: title.to_string(),
        current_locale: locale.code(),
        navbar_html,
        section: section.to_string(),
        sections,
        content_html,
        flash,
        flash_error,
    };
    Ok(Html(render(page)).into_response())
}

// ---------------------------------------------------------------------
// Moderation queues
// ---------------------------------------------------------------------

struct TabLink {
    key: &'static str,
    label: String,
    count: i64,
}

struct QueueRowView {
    id: i64,
    title: String,
    subtitle: Option<String>,
    author: Option<String>,
    at_human: String,
    at_iso: String,
    /// Where "view" goes — the existing public/edit page for that entity.
    /// `None` for images, which have no page of their own.
    view_path: Option<String>,
}

struct DiffView {
    field: String,
    old: String,
    new: String,
}

struct ChangeRowView {
    log_id: i64,
    title: String,
    author: Option<String>,
    at_human: String,
    at_iso: String,
    diff: Vec<DiffView>,
}

#[derive(Template)]
#[template(path = "admin/queue.html")]
struct QueueTemplate {
    slug: &'static str,
    heading: String,
    blurb: String,
    tab: String,
    tabs: Vec<TabLink>,
    rows: Vec<QueueRowView>,
    changes: Vec<ChangeRowView>,
}

#[derive(Deserialize)]
pub struct TabQuery {
    #[serde(default)]
    tab: Option<String>,
    /// Fluent key of a message to show above the queue, carried across
    /// the POST-then-redirect. A key rather than the text itself so the
    /// message is translated on render — the redirect target may well be
    /// read in the other language.
    #[serde(default)]
    error: Option<String>,
}

fn view_path(entity: Entity, id: i64) -> Option<String> {
    match entity {
        Entity::Company => Some(format!("/company/{id}/edit")),
        Entity::Store => Some(format!("/store/{id}")),
        Entity::Product => Some(format!("/product/{id}/edit")),
        Entity::Offer => Some(format!("/store-product/{id}/edit")),
        // No page shows a single image on its own; the bytes are at
        // /image/{id} but that's a download, not something to review in.
        Entity::Image => None,
    }
}

pub async fn queue(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(slug): Path<String>,
    Query(q): Query<TabQuery>,
) -> AppResult<Response> {
    let entity = Entity::from_slug(&slug).ok_or(AppError::NotFound)?;
    let locale = i18n::current_locale();
    let counts = db::moderation::counts(&state.pool, entity).await?;
    let tab = q.tab.unwrap_or_else(|| "pending".to_string());

    let tabs = vec![
        TabLink { key: "pending", label: i18n::translate(locale, "admin-tab-pending"), count: counts.pending },
        TabLink { key: "changes", label: i18n::translate(locale, "admin-tab-changes"), count: counts.changes },
        TabLink { key: "deleted", label: i18n::translate(locale, "admin-tab-deleted"), count: counts.deleted },
    ];

    // Only the visible tab is queried — the other two are just numbers,
    // which `counts` already has.
    let (rows, changes) = match tab.as_str() {
        "changes" => {
            let raw = db::moderation::changes(&state.pool, entity).await?;
            let changes = raw
                .into_iter()
                .map(|c| ChangeRowView {
                    log_id: c.log_id,
                    title: c.title,
                    author: c.author,
                    at_human: human(c.at),
                    at_iso: iso(c.at),
                    diff: c
                        .diff
                        .into_iter()
                        .map(|d| DiffView { field: d.field, old: d.old, new: d.new })
                        .collect(),
                })
                .collect();
            (Vec::new(), changes)
        }
        "deleted" => {
            let raw = db::moderation::deleted(&state.pool, entity).await?;
            (to_views(entity, raw), Vec::new())
        }
        _ => {
            let raw = db::moderation::pending(&state.pool, entity).await?;
            (to_views(entity, raw), Vec::new())
        }
    };

    let content = render(QueueTemplate {
        slug: entity.slug(),
        heading: i18n::translate(locale, entity.label_key()),
        blurb: i18n::translate(locale, blurb_key(entity)),
        tab: if tab == "changes" || tab == "deleted" { tab } else { "pending".to_string() },
        tabs,
        rows,
        changes,
    });

    // Only keys this module emits are accepted, so a crafted `?error=`
    // can't turn the banner into an arbitrary-string echo.
    let flash_error = q
        .error
        .filter(|key| key == "admin-error-name-taken")
        .map(|key| i18n::translate(locale, &key));

    shell(
        &state,
        &admin,
        &i18n::translate(locale, entity.label_key()),
        entity.slug(),
        content,
        None,
        flash_error,
    )
    .await
}

fn to_views(entity: Entity, rows: Vec<db::moderation::QueueRow>) -> Vec<QueueRowView> {
    rows.into_iter()
        .map(|r| QueueRowView {
            id: r.id,
            title: r.title,
            subtitle: r.subtitle,
            author: r.author,
            at_human: human(r.at),
            at_iso: iso(r.at),
            view_path: view_path(entity, r.id),
        })
        .collect()
}

fn blurb_key(entity: Entity) -> &'static str {
    match entity {
        Entity::Company => "admin-blurb-companies",
        Entity::Store => "admin-blurb-stores",
        Entity::Product => "admin-blurb-products",
        Entity::Offer => "admin-blurb-offers",
        Entity::Image => "admin-blurb-images",
    }
}

// ---------------------------------------------------------------------
// Moderation actions — POST, then redirect back to the queue
// ---------------------------------------------------------------------

/// The three queue actions share a shape: run it, then redirect back to
/// the tab it was launched from — where the row has just disappeared,
/// which is the confirmation. Written out three times rather than behind
/// a function-pointer indirection; the repetition is three lines each and
/// reads better than the machinery to remove it.
fn back_to(slug: &str, tab: &str, outcome: Outcome) -> Response {
    match outcome {
        Outcome::Done => Redirect::to(&format!("/admin/{slug}?tab={tab}")).into_response(),
        // The one restore that can fail: a product whose name was taken
        // by a new row while this one was deleted.
        Outcome::NameTaken(name) => {
            tracing::info!(taken_by = %name, "restore blocked: the name is in use again");
            Redirect::to(&format!("/admin/{slug}?tab={tab}&error=admin-error-name-taken"))
                .into_response()
        }
    }
}

pub async fn approve(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path((slug, id)): Path<(String, i64)>,
) -> AppResult<Response> {
    let entity = Entity::from_slug(&slug).ok_or(AppError::NotFound)?;
    let outcome = db::moderation::approve(&state.pool, entity, id, admin.id).await?;
    tracing::info!(entity = entity.table(), id, "approved");
    Ok(back_to(&slug, "pending", outcome))
}

pub async fn reject(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path((slug, id)): Path<(String, i64)>,
) -> AppResult<Response> {
    let entity = Entity::from_slug(&slug).ok_or(AppError::NotFound)?;
    let outcome = db::moderation::reject(&state.pool, entity, id, admin.id).await?;
    tracing::info!(entity = entity.table(), id, "rejected");
    Ok(back_to(&slug, "pending", outcome))
}

pub async fn restore(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path((slug, id)): Path<(String, i64)>,
) -> AppResult<Response> {
    let entity = Entity::from_slug(&slug).ok_or(AppError::NotFound)?;
    let outcome = db::moderation::restore(&state.pool, entity, id, admin.id).await?;
    tracing::info!(entity = entity.table(), id, "restored");
    Ok(back_to(&slug, "deleted", outcome))
}

pub async fn revert(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path((slug, log_id)): Path<(String, i64)>,
) -> AppResult<Response> {
    let entity = Entity::from_slug(&slug).ok_or(AppError::NotFound)?;
    db::moderation::revert(&state.pool, entity, log_id, admin.id).await?;
    Ok(Redirect::to(&format!("/admin/{slug}?tab=changes")).into_response())
}

// ---------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------

struct UserRowView {
    id: i64,
    name: String,
    email: String,
    admin: bool,
    contributions: i64,
    created_human: String,
    /// False for the row of the admin doing the looking, and for the last
    /// remaining admin — demoting either locks the door from inside.
    can_toggle_admin: bool,
    can_delete: bool,
    confirming_delete: bool,
    delete_warning: String,
}

#[derive(Template)]
#[template(path = "admin/users.html")]
struct UsersTemplate {
    users: Vec<UserRowView>,
}

#[derive(Deserialize)]
pub struct UsersQuery {
    #[serde(default)]
    confirm: Option<i64>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    ok: Option<String>,
}

pub async fn users(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Query(q): Query<UsersQuery>,
) -> AppResult<Response> {
    let locale = i18n::current_locale();
    let rows = db::user::list_all(&state.pool).await?;
    let admin_count = db::user::admin_count(&state.pool).await?;

    let users = rows
        .into_iter()
        .map(|u| {
            let is_self = u.id == admin.id;
            let is_last_admin = u.admin && admin_count <= 1;
            // The seeded account is the documented way back in after a
            // lockout, so it keeps its rights and its row.
            let is_seed = u.email.eq_ignore_ascii_case(SEED_ADMIN_EMAIL);
            UserRowView {
                can_toggle_admin: !is_self && !is_last_admin && !is_seed,
                can_delete: !is_self && !is_last_admin && !is_seed,
                confirming_delete: q.confirm == Some(u.id),
                delete_warning: i18n::translate_with_count(
                    locale,
                    "admin-users-delete-warning",
                    u.contributions,
                ),
                id: u.id,
                name: u.name,
                email: u.email,
                admin: u.admin,
                contributions: u.contributions,
                created_human: human_day(u.created),
            }
        })
        .collect();

    let content = render(UsersTemplate { users });
    let flash = q.ok.map(|key| i18n::translate(locale, &key));
    let flash_error = q.error.map(|key| i18n::translate(locale, &key));
    shell(
        &state,
        &admin,
        &i18n::translate(locale, "admin-nav-users"),
        "users",
        content,
        flash,
        flash_error,
    )
    .await
}

#[derive(Deserialize)]
pub struct NewUserForm {
    name: String,
    email: String,
    password: String,
    #[serde(default)]
    admin: Option<String>,
}

pub async fn create_user(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
    Form(form): Form<NewUserForm>,
) -> AppResult<Response> {
    let name = form.name.trim();
    let email = form.email.trim().to_lowercase();

    // Same gates as public registration — an account created here is a
    // real account, and the admin form is not a way around the policy.
    if name.is_empty() || !super::account::valid_email(&email) {
        return Ok(Redirect::to("/admin/users?error=admin-users-error-invalid").into_response());
    }
    if db::user::email_exists(&state.pool, &email).await? {
        return Ok(Redirect::to("/admin/users?error=admin-users-error-exists").into_response());
    }
    if password::check_policy(&form.password, name, &email).is_err() {
        return Ok(Redirect::to("/admin/users?error=admin-users-error-password").into_response());
    }

    let hash = password::hash_password(&form.password).map_err(AppError::from)?;
    let user = db::user::insert(&state.pool, name, &email, &hash).await?;
    if form.admin.is_some() {
        db::user::set_admin(&state.pool, user.id, true).await?;
    }
    tracing::info!(user_id = %user.id, "admin created user");
    Ok(Redirect::to("/admin/users?ok=admin-users-created").into_response())
}

#[derive(Deserialize)]
pub struct SetAdminForm {
    admin: String,
}

pub async fn set_admin(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<i64>,
    Form(form): Form<SetAdminForm>,
) -> AppResult<Response> {
    let make_admin = form.admin == "1";
    // Re-checked here, not just hidden in the template: the button is
    // absent for these cases, but a POST doesn't have to come from the
    // button.
    if let Some(redirect) = guard_self_or_last_admin(&state, &admin, id, make_admin).await? {
        return Ok(redirect);
    }
    db::user::set_admin(&state.pool, id, make_admin).await?;
    tracing::info!(target_user = %id, admin = make_admin, "admin role changed");
    Ok(Redirect::to("/admin/users?ok=admin-users-role-changed").into_response())
}

pub async fn delete_user(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    if let Some(redirect) = guard_self_or_last_admin(&state, &admin, id, false).await? {
        return Ok(redirect);
    }
    db::user::delete(&state.pool, id).await?;
    tracing::info!(target_user = %id, "admin deleted user");
    Ok(Redirect::to("/admin/users?ok=admin-users-deleted").into_response())
}

/// Blocks the two ways an admin can lock themselves (or everyone) out:
/// acting on their own row, and removing the rights of the last admin.
/// Returns the redirect to send instead, or `None` to proceed.
async fn guard_self_or_last_admin(
    state: &AppState,
    admin: &crate::models::User,
    target_id: i64,
    make_admin: bool,
) -> AppResult<Option<Response>> {
    if target_id == admin.id {
        return Ok(Some(
            Redirect::to("/admin/users?error=admin-users-error-self").into_response(),
        ));
    }
    let target = db::user::find_by_id(&state.pool, target_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if target.email.eq_ignore_ascii_case(SEED_ADMIN_EMAIL) {
        return Ok(Some(
            Redirect::to("/admin/users?error=admin-users-error-seed").into_response(),
        ));
    }
    if target.admin && !make_admin && db::user::admin_count(&state.pool).await? <= 1 {
        return Ok(Some(
            Redirect::to("/admin/users?error=admin-users-error-last-admin").into_response(),
        ));
    }
    Ok(None)
}

// ---------------------------------------------------------------------
// Site info (the Impressum's contents)
// ---------------------------------------------------------------------

#[derive(Template)]
#[template(path = "admin/site_info.html")]
struct SiteInfoTemplate {
    info: db::site_info::SiteInfo,
}

#[derive(Deserialize)]
pub struct SiteInfoQuery {
    #[serde(default)]
    ok: Option<String>,
}

pub async fn site_info(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Query(q): Query<SiteInfoQuery>,
) -> AppResult<Response> {
    let locale = i18n::current_locale();
    let info = db::site_info::get(&state.pool).await?;
    let content = render(SiteInfoTemplate { info });
    let flash = q
        .ok
        .filter(|key| key == "admin-site-info-saved")
        .map(|key| i18n::translate(locale, &key));
    shell(
        &state,
        &admin,
        &i18n::translate(locale, "admin-nav-site-info"),
        "site-info",
        content,
        flash,
        None,
    )
    .await
}

/// Every field is optional and free-form on purpose: an Impressum's
/// contents are whatever the operator is legally required to state, and
/// that differs by country and by whether they're a business. The one
/// thing the app cares about is `operator_name`, and only to decide
/// whether the public page has anything worth showing.
#[derive(Deserialize)]
pub struct SiteInfoForm {
    #[serde(default)]
    operator_name: String,
    #[serde(default)]
    street: String,
    #[serde(default)]
    postal_code: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    phone: String,
    #[serde(default)]
    vat_id: String,
    #[serde(default)]
    register_number: String,
    #[serde(default)]
    responsible: String,
    #[serde(default)]
    purpose: String,
}

pub async fn save_site_info(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Form(form): Form<SiteInfoForm>,
) -> AppResult<Response> {
    let info = db::site_info::SiteInfo {
        operator_name: form.operator_name,
        street: form.street,
        postal_code: form.postal_code,
        city: form.city,
        country: form.country,
        email: form.email,
        phone: form.phone,
        vat_id: form.vat_id,
        register_number: form.register_number,
        responsible: form.responsible,
        purpose: form.purpose,
    };
    db::site_info::update(&state.pool, &info, admin.id).await?;
    tracing::info!(admin_id = %admin.id, "site info updated");
    Ok(Redirect::to("/admin/site-info?ok=admin-site-info-saved").into_response())
}

/// `/admin` with no section — land on the first queue that has work, or
/// on users if everything is clear.
pub async fn index(
    State(state): State<AppState>,
    AdminUser(_admin): AdminUser,
) -> AppResult<Response> {
    for entity in Entity::ALL {
        if db::moderation::counts(&state.pool, entity).await?.pending > 0 {
            return Ok(Redirect::to(&format!("/admin/{}", entity.slug())).into_response());
        }
    }
    Ok(Redirect::to("/admin/users").into_response())
}
