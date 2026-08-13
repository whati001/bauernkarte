//! user-auth capability: register, login, logout, account edit. Every
//! mutation here is anon-only or self-only — none of it is touched by
//! catalog-editing's "any logged-in user" rule (that's scoped to catalog
//! entities, not accounts; see catalog-editing spec's scope requirement).

use askama::Template;
use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream;
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use tower_sessions::Session;

use crate::{
    auth::{self, password, CurrentUser, OptionalUser},
    db,
    error::{AppError, AppResult},
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    sse::{patch_elements_at, patch_signals},
    state::AppState,
    templates::render,
};

fn t_arg(key: &str, name: &str) -> String {
    i18n::translate_with_name(i18n::current_locale(), key, name)
}

fn valid_email(email: &str) -> bool {
    let email = email.trim();
    // Deliberately loose per design.md's "plausible email address" bar —
    // full RFC 5322 validation is out of scope, this just rejects the
    // obviously-malformed.
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

#[derive(Template)]
#[template(path = "partials/auth_login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "partials/auth_register.html")]
struct RegisterTemplate;

pub fn render_login() -> String {
    render(LoginTemplate)
}

pub fn render_register() -> String {
    render(RegisterTemplate)
}

/// `GET /login` — anon-only form fragment (design.md route table). Used
/// both for the fragment endpoint and reused by `pages.rs` if it's ever
/// linked as a full page.
pub async fn login_form(
    OptionalUser(user): OptionalUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    if user.is_some() {
        return Err(AppError::Conflict("already logged in".into()));
    }
    let html = render_login();
    Ok(Sse::new(stream::iter(vec![
        // `email`/`password` are the same signal *names* the register and
        // account forms bind to (`data-bind:email`/`data-bind:password`)
        // — with no reset, Datastar happily carries over whatever was
        // last typed into either of those forms and silently re-fills
        // this one with it on mount (caught via a real browser: the
        // fields *looked* empty per the server-rendered `value=""`, but
        // already held the old value the moment the page applied the
        // patch). Password is worth clearing on its own merits too —
        // no reason a stale plaintext value should hang around in the
        // client's signal store past the form it was typed into.
        Ok(patch_signals(&json!({ "email": "", "password": "" }))),
        Ok(patch_elements_at("#sidebar", "inner", &html)),
    ])))
}

pub async fn register_form(
    OptionalUser(user): OptionalUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    if user.is_some() {
        return Err(AppError::Conflict("already logged in".into()));
    }
    let html = render_register();
    Ok(Sse::new(stream::iter(vec![
        // See login_form's comment — same cross-form signal leakage, plus
        // `name` here (also reused by the account page's own `name` field).
        Ok(patch_signals(&json!({ "name": "", "email": "", "password": "" }))),
        Ok(patch_elements_at("#sidebar", "inner", &html)),
    ])))
}

#[derive(Deserialize)]
pub struct RegisterBody {
    name: String,
    email: String,
    password: String,
}

/// `POST /register` — creates the account (`verified=false`), auto-logs
/// in (user-auth capability).
pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<RegisterBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let name = body.name.trim();
    let email = body.email.trim().to_lowercase();

    if name.is_empty() {
        return Err(AppError::Validation("Bitte einen Namen angeben.".into()));
    }
    if !valid_email(&email) {
        return Err(AppError::Validation("Bitte eine gültige E-Mail-Adresse angeben.".into()));
    }
    if body.password.len() < 8 {
        return Err(AppError::Validation("Das Passwort muss mindestens 8 Zeichen haben.".into()));
    }
    if db::user::email_exists(&state.pool, &email).await? {
        return Err(AppError::Validation("Diese E-Mail-Adresse ist bereits registriert.".into()));
    }

    let pwd_hash = password::hash_password(&body.password).map_err(AppError::from)?;
    let user = db::user::insert(&state.pool, name, &email, &pwd_hash).await?;
    auth::log_in(&session, user.id).await.map_err(AppError::from)?;
    tracing::info!(user_id = %user.id, "user registered");

    let navbar_html = crate::templates::render_navbar(Some(user.name.clone()));
    // Previously this only patched #navbar, leaving the now-stale
    // registration form on screen with no visible confirmation beyond
    // the navbar changing — easy to miss. Now also swaps #sidebar to a
    // confirmation with a way back to search.
    let sidebar_html =
        crate::templates::render_confirmation(&t_arg("auth-register-success", &user.name));
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_signals(&json!({ "loggedIn": true }))),
        Ok(patch_elements_at("#navbar", "outer", &navbar_html)),
        Ok(patch_elements_at("#sidebar", "inner", &sidebar_html)),
    ])))
}

#[derive(Deserialize)]
pub struct LoginBody {
    email: String,
    password: String,
}

/// `POST /login` (user-auth capability).
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<LoginBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let email = body.email.trim().to_lowercase();
    let user = db::user::find_by_email(&state.pool, &email).await?;
    let Some(user) = user else {
        // The identifier attempted (never the password) — worth WARN, not
        // just a 200-with-form-error: a run of these against one address
        // is exactly what a brute-force/credential-stuffing attempt looks
        // like, and login is otherwise the one route with no other audit
        // trail (unlike catalog mutations, which also land in edit_log).
        tracing::warn!(email = %email, "login failed: unknown email");
        return Err(AppError::Validation("E-Mail oder Passwort ist falsch.".into()));
    };
    if !password::verify_password(&body.password, &user.pwd_hash) {
        tracing::warn!(user_id = %user.id, "login failed: wrong password");
        return Err(AppError::Validation("E-Mail oder Passwort ist falsch.".into()));
    }

    auth::log_in(&session, user.id).await.map_err(AppError::from)?;
    tracing::info!(user_id = %user.id, "user logged in");

    let navbar_html = crate::templates::render_navbar(Some(user.name.clone()));
    let sidebar_html = crate::templates::render_confirmation(&t_arg("auth-welcome-back", &user.name));
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_signals(&json!({ "loggedIn": true }))),
        Ok(patch_elements_at("#navbar", "outer", &navbar_html)),
        Ok(patch_elements_at("#sidebar", "inner", &sidebar_html)),
    ])))
}

/// `POST /logout` (user-auth capability).
pub async fn logout(session: Session) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    auth::log_out(&session).await.map_err(AppError::from)?;
    let navbar_html = crate::templates::render_navbar(None);
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_signals(&json!({ "loggedIn": false }))),
        Ok(patch_elements_at("#navbar", "outer", &navbar_html)),
    ])))
}

#[derive(Template)]
#[template(path = "partials/account.html")]
struct AccountTemplate {
    name: String,
    email: String,
    pending: Vec<db::pending::PendingItem>,
    success_message: Option<String>,
}

/// `GET /account` — view/update own profile; also lists the user's
/// pending (`NOT approved`) submissions (content-moderation capability).
pub async fn account_page(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let pending = db::pending::for_user(&state.pool, user.id).await?;
    let html = render(AccountTemplate {
        name: user.name.clone(),
        email: user.email.clone(),
        pending,
        success_message: None,
    });
    // Explicitly (re)seed $name/$email so the bound inputs reflect the
    // current account even if the client's signal store had stale/empty
    // values from a previous page state.
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_signals(&json!({ "name": user.name, "email": user.email }))),
        Ok(patch_elements_at("#sidebar", "inner", &html)),
    ])))
}

#[derive(Deserialize)]
pub struct UpdateProfileBody {
    name: String,
    email: String,
}

/// `POST /account` — update own `name`/`email` (user-auth capability).
pub async fn update_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<UpdateProfileBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let name = body.name.trim();
    let email = body.email.trim().to_lowercase();
    if name.is_empty() {
        return Err(AppError::Validation("Bitte einen Namen angeben.".into()));
    }
    if !valid_email(&email) {
        return Err(AppError::Validation("Bitte eine gültige E-Mail-Adresse angeben.".into()));
    }

    let updated = db::user::update_profile(&state.pool, user.id, name, &email).await?;
    tracing::info!(user_id = %user.id, "profile updated");
    let pending = db::pending::for_user(&state.pool, user.id).await?;
    let html = render(AccountTemplate {
        name: updated.name.clone(),
        email: updated.email.clone(),
        pending,
        success_message: Some(i18n::translate(i18n::current_locale(), "account-profile-saved")),
    });
    let navbar_html = crate::templates::render_navbar(Some(updated.name.clone()));
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &html)),
        Ok(patch_elements_at("#navbar", "outer", &navbar_html)),
    ])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordBody {
    current_password: String,
    new_password: String,
}

/// `POST /account/password` — change password, requires the current one
/// (user-auth capability).
pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<ChangePasswordBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    // `ValidationAt`, not `Validation`: this page's profile form has its
    // own `#form-error` right above this one — a plain `Validation`
    // would morph *that* slot instead of this form's `#password-form-error`.
    if !password::verify_password(&body.current_password, &user.pwd_hash) {
        tracing::warn!(user_id = %user.id, "password change rejected: wrong current password");
        return Err(AppError::ValidationAt("Aktuelles Passwort ist falsch.".into(), "password-form-error"));
    }
    if body.new_password.len() < 8 {
        return Err(AppError::ValidationAt(
            "Das neue Passwort muss mindestens 8 Zeichen haben.".into(),
            "password-form-error",
        ));
    }
    let new_hash = password::hash_password(&body.new_password).map_err(AppError::from)?;
    db::user::update_password(&state.pool, user.id, &new_hash).await?;
    tracing::info!(user_id = %user.id, "password changed");

    let pending = db::pending::for_user(&state.pool, user.id).await?;
    let html = render(AccountTemplate {
        name: user.name.clone(),
        email: user.email.clone(),
        pending,
        success_message: Some(i18n::translate(i18n::current_locale(), "account-password-changed")),
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}
