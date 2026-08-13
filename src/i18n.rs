//! DE/EN translations via Fluent (`.ftl` files in `locales/`, loaded at
//! compile time by `fluent_templates::static_loader!`) and a custom
//! Askama filter (`|t`) that reads the current request's locale through
//! Askama's `render_with_values` side-channel — confirmed against the
//! askama 0.16 source (`askama::Values`/`get_value`/`render_with_values`,
//! and the `#[filter_fn]` signature convention
//! `fn f(_: &dyn Display, _: &dyn askama::Values) -> askama::Result<String>`)
//! rather than guessed, since Askama itself has no official i18n story
//! and this crate combination isn't documented anywhere as a pair.
//!
//! This avoids threading a `locale` field through every template struct
//! (~20 of them): `templates::render()` passes the current locale once,
//! as a value, and any template can pull a translated string with
//! `{{ "key"|t }}` — same render call, no struct changes needed per
//! string.

use askama::Values;
use axum::{extract::Request, http::header, middleware::Next, response::Response};
use fluent_templates::{static_loader, Loader};
use std::collections::HashMap;
use unic_langid::{langid, LanguageIdentifier};

tokio::task_local! {
    /// Set once per request by the locale-resolution middleware
    /// (`main.rs`), read by `templates::render()` so every render call
    /// gets the right language without threading a `locale` parameter
    /// through the ~20 template structs and their handlers individually.
    /// A `tokio::task_local!`, not a plain `thread_local!`, because a
    /// single OS thread interleaves many concurrent requests' futures —
    /// only a task-scoped local stays correctly isolated per request.
    pub static CURRENT_LOCALE: Locale;
}

/// Falls back to German outside a request context (e.g. a unit test
/// that renders a template directly).
pub fn current_locale() -> Locale {
    CURRENT_LOCALE.try_with(|l| *l).unwrap_or(Locale::De)
}

pub const LOCALE_COOKIE: &str = "locale";

/// Reads the `locale` cookie (set by `GET /locale/{code}`, see
/// `handlers::locale`) and makes it available to every `render()` call
/// for the duration of this request via `CURRENT_LOCALE`. Applied as a
/// blanket `axum::middleware::from_fn` layer in `main.rs` — every route
/// needs it, including pages that don't otherwise touch locale-specific
/// logic, so a per-route extractor would just mean repeating it
/// everywhere for no benefit.
pub async fn locale_middleware(request: Request, next: Next) -> Response {
    let locale = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|kv| {
                let (k, v) = kv.trim().split_once('=')?;
                (k == LOCALE_COOKIE).then(|| v.to_string())
            })
        })
        .map(|code| Locale::from_code(&code))
        .unwrap_or(Locale::De);

    CURRENT_LOCALE.scope(locale, next.run(request)).await
}

static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "de",
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    De,
    En,
}

impl Locale {
    pub fn langid(self) -> LanguageIdentifier {
        match self {
            Locale::De => langid!("de"),
            Locale::En => langid!("en"),
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Locale::De => "de",
            Locale::En => "en",
        }
    }

    /// Unrecognized/missing cookie values fall back to German — the
    /// app's original language, not an arbitrary default.
    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Locale::En,
            _ => Locale::De,
        }
    }
}

/// For handler code building a message string directly — e.g. a
/// `success_message` field — rather than a static label a template pulls
/// via `{{ "key"|t }}`. Named `translate`, not `t`: the latter is already
/// the askama filter function below, and Rust doesn't allow two `fn t` in
/// one module regardless of arity.
pub fn translate(locale: Locale, key: &str) -> String {
    LOCALES.lookup(&locale.langid(), key)
}

/// The common case of `translate_with_args` — a message with exactly one
/// `{ $name }` placeholder (confirmation panels, welcome messages).
pub fn translate_with_name(locale: Locale, key: &str, name: &str) -> String {
    let mut args = HashMap::new();
    args.insert("name".to_string(), name.to_string());
    translate_with_args(locale, key, &args)
}

/// As `translate`, with Fluent message arguments (e.g. `{ $name }`).
pub fn translate_with_args(locale: Locale, key: &str, args: &HashMap<String, String>) -> String {
    let fluent_args: HashMap<std::borrow::Cow<'static, str>, fluent_templates::fluent_bundle::FluentValue<'static>> = args
        .iter()
        .map(|(k, v)| (std::borrow::Cow::Owned(k.clone()), fluent_templates::fluent_bundle::FluentValue::String(std::borrow::Cow::Owned(v.clone()))))
        .collect();
    LOCALES.lookup_with_args(&locale.langid(), key, &fluent_args)
}

/// The `{{ "key"|t }}` template filter. Reads `"locale"` out of whatever
/// `Values` the current `render_with_values()` call supplied; falls back
/// to German if none was provided (e.g. a template rendered directly via
/// `.render()` in a test, bypassing `templates::render()`).
#[askama::filter_fn]
pub fn t(key: &str, values: &dyn Values) -> askama::Result<String> {
    let locale = askama::get_value::<Locale>(values, "locale")
        .copied()
        .unwrap_or(Locale::De);
    Ok(LOCALES.lookup(&locale.langid(), key))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fails loudly at test time (not silently at render time) if the
    /// two catalogs drift — Fluent's own fallback would otherwise just
    /// quietly show German text on an English page for a missing key.
    #[test]
    fn de_and_en_have_the_same_keys() {
        let de = langid!("de");
        let en = langid!("en");
        // fluent-templates doesn't expose a direct "list all keys" API,
        // so this walks the known key list instead — the single place
        // that has to be kept honest as keys are added.
        for key in ALL_KEYS {
            let de_val = LOCALES.lookup(&de, key);
            let en_val = LOCALES.lookup(&en, key);
            assert_ne!(de_val, *key, "missing German translation for {key}");
            assert_ne!(en_val, *key, "missing English translation for {key}");
        }
    }

    const ALL_KEYS: &[&str] = &[
        "nav-brand", "nav-login", "nav-logout", "nav-account", "nav-new-store",
        "search-category", "search-product", "search-distance", "search-all",
        "search-no-results", "search-more", "search-location-unavailable", "search-pick-on-map",
        "search-location-picked", "search-use-my-location",
        "action-save", "action-delete", "action-cancel", "action-edit", "action-back",
        "action-back-to-search",
        "auth-login-heading", "auth-register-heading", "auth-email", "auth-password",
        "auth-password-hint", "auth-name", "auth-no-account", "auth-have-account",
        "account-heading", "account-change-password", "account-current-password",
        "account-new-password", "account-pending-heading", "account-pending-empty",
        "account-profile-saved", "account-password-changed",
        "detail-open-in-maps", "detail-add-product", "detail-add-image", "detail-rate",
        "detail-unrate", "detail-no-products",
        "store-form-new-heading", "store-form-edit-heading", "store-form-name",
        "store-form-location", "store-form-opening-hours", "store-form-is-company",
        "store-form-company", "store-form-company-choose", "store-form-company-description",
        "store-form-company-homepage", "store-form-product-heading",
        "company-form-heading", "company-form-name", "company-form-description",
        "company-form-homepage",
        "product-form-add-heading", "product-form-new-checkbox", "product-form-product",
        "product-form-choose", "product-form-name", "product-form-category",
        "product-form-description-optional",
        "edit-product-form-heading", "edit-product-form-name", "edit-product-form-category",
        "edit-product-form-description",
        "image-form-heading", "image-form-file-label", "image-form-description-optional",
        "image-form-upload", "image-form-alt-fallback",
        "detail-edit-company", "detail-edit-store", "detail-delete-store",
        "detail-edit-product", "detail-edit-product-title",
        "detail-remove-offer", "detail-remove-offer-title",
        "confirmation-image-pending",
        "map-sidebar-collapse", "map-sidebar-expand",
        "language-de", "language-en",
    ];
}
