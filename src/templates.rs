//! `askama_axum` was removed from askama as of 0.13 (integration crates
//! deprecated upstream); this is the replacement glue — render a template
//! to a `String` and hand it to axum as HTML. Datastar SSE responses
//! (`patch-elements`/`patch-signals`) render templates the same way, then
//! wrap the string in an SSE event (see `sse.rs`), so this one helper
//! covers both full-page and fragment rendering.

// Askama's generated code for a custom filter (`{{ "key"|t }}`) emits a
// literal `filters::t(...)` call — it expects a module named exactly
// `filters` in scope, mirroring how its own built-ins live under
// `askama::filters`. Aliasing this crate's `i18n` module (which defines
// `t`) to that name is what satisfies it; this alias has to be repeated
// in every module that derives `Template` for a template using `|t`,
// since Askama resolves it via normal Rust path lookup in the struct's
// own module, not a global registry.
use crate::i18n;
use crate::i18n as filters;
use crate::models::RankedProduct;
use askama::{Template, Values};
use axum::response::Html;
use serde_json::Value;

/// Render a template to a `String`, falling back to an empty string with a
/// logged error rather than panicking — a template render failure should
/// never take the whole request down.
///
/// Every call goes through `render_with_values` carrying the current
/// request's locale (`i18n::current_locale()`, set per-request by the
/// locale-resolution middleware in `main.rs`) under the key `"locale"` —
/// this is what the `{{ "key"|t }}` filter in `i18n::t` reads. One choke
/// point here means no template struct needs its own `locale` field.
pub fn render<T: Template>(template: T) -> String {
    let locale = i18n::current_locale();
    let values: [(&str, &dyn std::any::Any); 1] = [("locale", &locale)];
    match template.render_with_values(&LocaleValues(&values)) {
        Ok(html) => html,
        Err(err) => {
            tracing::error!(error = %err, "template render failed");
            String::new()
        }
    }
}

struct LocaleValues<'a>(&'a [(&'a str, &'a dyn std::any::Any)]);

impl Values for LocaleValues<'_> {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        self.0.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

#[derive(Template)]
#[template(path = "partials/confirmation.html")]
struct ConfirmationTemplate<'a> {
    message: &'a str,
}

/// A generic "✓ <message>, here's a way back to search" panel — used for
/// the community-submissions "wird geprüft" confirmation (design.md's
/// approval-workflow sequence diagram) and for post-login/register
/// feedback, so a successful auth action doesn't just silently leave the
/// stale form on screen.
pub fn render_confirmation(message: &str) -> String {
    render(ConfirmationTemplate { message })
}

#[derive(Template)]
#[template(path = "partials/navbar.html")]
struct NavbarTemplate {
    user_name: Option<String>,
    current_locale: &'static str,
    /// Always the *empty* dropdown — `#nav-suggestions` only has to exist
    /// in the DOM for `GET /api/search/suggest` to patch it by id.
    /// Embedded as pre-rendered HTML (same deal as `full_page`'s
    /// `sidebar_html`) so the container's markup lives in exactly one
    /// template, shared with the patch path.
    suggestions_html: String,
    nav_products: Vec<RankedProduct>,
}

/// `products` is the quick-pick row (`db::product::list_top_rated`).
/// Every caller has to supply it, including the ones that re-render the
/// navbar just to flip the login state — the row lives inside `#navbar`,
/// so a `patch-elements #navbar` built without it would silently delete
/// the row on login/logout.
pub fn render_navbar(user_name: Option<String>, products: Vec<RankedProduct>) -> String {
    render(NavbarTemplate {
        user_name,
        current_locale: i18n::current_locale().code(),
        suggestions_html: crate::handlers::search::render_empty_suggestions(),
        nav_products: products,
    })
}

#[derive(Template)]
#[template(path = "layout.html")]
struct LayoutTemplate {
    title: String,
    signals_json: String,
    navbar_html: String,
    sidebar_html: String,
    map_data_html: String,
    sidebar_collapsed: bool,
}

/// Every full-page GET (`/`, `/store/{id}`, and the anon-only auth pages)
/// goes through this: render the navbar + a pre-rendered sidebar fragment
/// into the shared shell. The same sidebar-fragment `String` this takes is
/// exactly what the matching `/api/...` SSE route sends as a
/// `patch-elements` body — one render path, two delivery mechanisms.
/// `map_data_html` is the same deal, one level down (`#map-data`, see
/// `handlers::search::render_map_data`) — every full-page GET populates
/// the map's pins up front too, deep links (`/store/{id}`) included, not
/// just the search landing page.
/// `sidebar_collapsed` is the panel's *initial* state only — map.js takes
/// over from there (`syncSidebarLifecycle`). `true` for the map-first
/// landing page, `false` for a `/store/{id}` deep link, whose whole
/// purpose is the panel's contents.
pub fn full_page(
    title: &str,
    user_name: Option<String>,
    signals: &Value,
    sidebar_html: String,
    map_data_html: String,
    sidebar_collapsed: bool,
    nav_products: Vec<RankedProduct>,
) -> Html<String> {
    let navbar_html = render_navbar(user_name, nav_products);
    let page = LayoutTemplate {
        title: title.to_string(),
        signals_json: signals.to_string(),
        navbar_html,
        sidebar_html,
        map_data_html,
        sidebar_collapsed,
    };
    Html(render(page))
}
