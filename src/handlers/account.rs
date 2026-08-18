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

/// Every handler here that re-renders `#navbar` (login/logout/profile
/// change) has to re-render its quick-pick product row along with it —
/// the row is inside `#navbar`, so a `patch-elements #navbar` built
/// without it would silently delete the row the moment someone logs in.
async fn nav_products(state: &AppState) -> AppResult<Vec<crate::models::RankedProduct>> {
    Ok(db::product::list_top_rated(&state.pool, crate::handlers::pages::NAV_PRODUCT_LIMIT).await?)
}

/// Characters RFC 5322 allows unquoted in the local part, beyond
/// alphanumerics. Quoted local parts (`"a b"@example.com`) are legal and
/// rejected here anyway — see the note on `valid_email`.
const LOCAL_SPECIALS: &str = ".!#$%&'*+/=?^_`{|}~-";

/// Whether `email` is a plausible, deliverable-looking address.
///
/// Still short of full RFC 5322 (per design.md's "plausible email
/// address" bar) and deliberately so: the grammar admits quoted local
/// parts, comments, IP-literal domains and other forms no real signup
/// uses, and accepting them buys nothing while widening what reaches the
/// database. What this *does* now reject, and the previous two-line
/// version didn't, is whitespace anywhere, several `@`s, empty or
/// oversized labels, consecutive dots, leading/trailing dots and
/// hyphens, and a numeric or single-character TLD.
///
/// **Mirrored in `static/credential-policy.js`** (`isValidEmail`) so the
/// live checkmark under the field agrees with what this accepts. Change
/// one, change the other; the tests below are the specification.
pub fn valid_email(email: &str) -> bool {
    let email = email.trim();
    // 254 is the practical ceiling on an SMTP forward path (RFC 5321
    // §4.5.3.1.3), which is the number that actually constrains a
    // deliverable address.
    if email.len() < 3 || email.len() > 254 {
        return false;
    }
    let mut parts = email.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    valid_local_part(local) && valid_domain(domain)
}

fn valid_local_part(local: &str) -> bool {
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
        return false;
    }
    local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || LOCAL_SPECIALS.contains(c))
}

fn valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 || !domain.contains('.') {
        return false;
    }
    let labels: Vec<&str> = domain.split('.').collect();
    let all_labels_well_formed = labels.iter().all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    });
    // A real TLD is alphabetic and at least two characters — this is what
    // turns away `user@host` (no dot at all is caught above), `a@b.c` and
    // `a@1.2.3.4`.
    let tld_looks_real = labels
        .last()
        .is_some_and(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()));
    all_labels_well_formed && tld_looks_real
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
    // The policy, not just a length floor — same rules the checklist
    // under the field shows live (auth/password.rs's `check_policy`).
    // Enforced here because the checklist is a hint: nothing stops a
    // client from posting whatever it likes.
    if let Err(rule) = password::check_policy(&body.password, name, &email) {
        return Err(AppError::Validation(
            i18n::translate(i18n::current_locale(), rule.message_key()),
        ));
    }
    if db::user::email_exists(&state.pool, &email).await? {
        return Err(AppError::Validation("Diese E-Mail-Adresse ist bereits registriert.".into()));
    }

    let pwd_hash = password::hash_password(&body.password).map_err(AppError::from)?;
    let user = db::user::insert(&state.pool, name, &email, &pwd_hash).await?;
    auth::log_in(&session, user.id).await.map_err(AppError::from)?;
    tracing::info!(user_id = %user.id, "user registered");

    let navbar_html = crate::templates::render_navbar(Some(&user), nav_products(&state).await?);
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

    let navbar_html = crate::templates::render_navbar(Some(&user), nav_products(&state).await?);
    let sidebar_html = crate::templates::render_confirmation(&t_arg("auth-welcome-back", &user.name));
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_signals(&json!({ "loggedIn": true }))),
        Ok(patch_elements_at("#navbar", "outer", &navbar_html)),
        Ok(patch_elements_at("#sidebar", "inner", &sidebar_html)),
    ])))
}

/// `POST /logout` (user-auth capability).
pub async fn logout(
    State(state): State<AppState>,
    session: Session,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    auth::log_out(&session).await.map_err(AppError::from)?;
    let navbar_html = crate::templates::render_navbar(None, nav_products(&state).await?);
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
    let navbar_html = crate::templates::render_navbar(Some(&updated), nav_products(&state).await?);
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
    // `user.name`/`user.email` are the stored values — the profile form
    // above saves separately, so what's in the database is what the new
    // password is checked against.
    if let Err(rule) = password::check_policy(&body.new_password, &user.name, &user.email) {
        return Err(AppError::ValidationAt(
            i18n::translate(i18n::current_locale(), rule.message_key()),
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

#[cfg(test)]
mod tests {
    use super::valid_email;

    #[test]
    fn accepts_ordinary_addresses() {
        for email in [
            "a@bc.de",
            "maxi.berg@example.com",
            "max+bauernkarte@sub.example.co.uk",
            "user_name-1@example-host.org",
            "  spaced@example.com  ", // trimmed before checking
        ] {
            assert!(valid_email(email), "should accept {email}");
        }
    }

    #[test]
    fn rejects_malformed_addresses() {
        for email in [
            "",
            "plainstring",
            "@example.com",
            "user@",
            "user@host",          // no dot in the domain
            "user@host.c",        // single-character TLD
            "user@1.2.3.4",       // numeric TLD
            "a@b..c.de",          // empty label
            "a@.example.com",
            "a@example.com.",
            "a@-example.com",
            "a@example-.com",
            "user name@example.com",
            "user@exa mple.com",
            "a@@example.com",
            "two@at@example.com",
            ".user@example.com",
            "user.@example.com",
            "us..er@example.com",
            "us,er@example.com",
        ] {
            assert!(!valid_email(email), "should reject {email:?}");
        }
    }

    #[test]
    fn rejects_oversized_parts() {
        let long_local = format!("{}@example.com", "a".repeat(65));
        assert!(!valid_email(&long_local));
        let long_label = format!("a@{}.com", "b".repeat(64));
        assert!(!valid_email(&long_label));
        let long_overall = format!("a@{}.com", vec!["b".repeat(60); 5].join("."));
        assert!(!valid_email(&long_overall));
    }
}
