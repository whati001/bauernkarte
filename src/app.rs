//! Routes and the app root: who is visiting and in which language, made
//! available to every component as context.

use dioxus::prelude::*;

use crate::{
    api,
    components::{
        account::{Account, Login, Register},
        admin::{AdminIndex, AdminQueue, AdminShell, AdminSiteInfo, AdminUsers},
        forms::{AddOffer, AddPhoto, EditOffer, EditProduct, EditStore, NewStore},
        impressum::Impressum,
        search::SearchPanel,
        shell::MapShell,
        store::StorePanel,
    },
    i18n::Locale,
    models::{SessionInfo, SessionUser},
    ui::{select::SelectPlaceholder, toast::ToastProvider, Stylesheets},
};

/// Every page with the map lives under `MapShell`, whose sidebar shows the
/// route's panel — so moving between search, a store and a form never
/// reloads the map. The admin area is its own full-page layout.
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MapShell)]
        #[route("/")]
        SearchPanel {},
        #[route("/store/:id")]
        StorePanel { id: i64 },
        #[route("/new-store")]
        NewStore {},
        #[route("/store/:id/edit")]
        EditStore { id: i64 },
        #[route("/store/:id/offer/new")]
        AddOffer { id: i64 },
        #[route("/store/:id/photo/new")]
        AddPhoto { id: i64 },
        #[route("/offer/:id/edit")]
        EditOffer { id: i64 },
        #[route("/store/:store_id/product/:id/edit")]
        EditProduct { store_id: i64, id: i64 },
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
        #[route("/account")]
        Account {},
        #[route("/impressum")]
        Impressum {},
    #[end_layout]
    #[layout(AdminShell)]
        #[route("/admin")]
        AdminIndex {},
        #[route("/admin/users")]
        AdminUsers {},
        #[route("/admin/site-info")]
        AdminSiteInfo {},
        #[route("/admin/queue/:slug?:tab")]
        AdminQueue { slug: String, tab: String },
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// The signed-in user, shared app-wide and updated in place on login,
/// logout and profile changes.
#[derive(Clone, Copy)]
pub struct Session(pub Signal<Option<SessionUser>>);

impl Session {
    pub fn user(&self) -> Option<SessionUser> {
        self.0.read().clone()
    }

    pub fn is_logged_in(&self) -> bool {
        self.0.read().is_some()
    }
}

pub fn use_session() -> Session {
    use_context::<Session>()
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover" }
        document::Meta { name: "theme-color", content: "#f5efe4" }
        document::Link { rel: "icon", r#type: "image/svg+xml", href: "/static/favicon.svg" }
        document::Link { rel: "manifest", href: "/static/manifest.webmanifest" }
        document::Link { rel: "apple-touch-icon", href: "/static/icons/apple-touch-icon.png" }
        document::Stylesheet { href: "/static/leaflet/leaflet.css" }
        document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
        document::Stylesheet { href: asset!("/assets/app.css") }
        Stylesheets {}
        // Deferred: neither blocks parsing, and `defer` keeps them in
        // order, so Leaflet is there when the bridge starts the map.
        document::Script { src: "/static/leaflet/leaflet.js", defer: true }
        document::Script { src: "/static/bk-map.js", defer: true }
        document::Script { src: "/static/pwa.js", defer: true }
        document::Script { src: "/static/tap-guard.js", defer: true }
        SuspenseBoundary {
            fallback: |_| rsx! {},
            SessionRoot {}
        }
    }
}

/// Resolved on the server before anything renders, so the HTML and the
/// hydrating client agree on who is logged in and in which language.
#[component]
fn SessionRoot() -> Element {
    let info = use_server_future(api::session::session_info)?;
    let info = info().and_then(Result::ok).unwrap_or(SessionInfo { user: None, locale: Locale::De });
    rsx! {
        Provide { info }
    }
}

#[component]
fn Provide(info: SessionInfo) -> Element {
    use_context_provider(|| info.locale);
    use_context_provider(|| SelectPlaceholder(info.locale.t("product-form-choose")));
    use_context_provider(|| Session(Signal::new(info.user.clone())));
    // `<html lang>` follows the visitor's language (screen readers and
    // browser translation read it).
    use_effect(move || {
        document::eval(&format!("document.documentElement.lang = '{}';", info.locale.code()));
    });
    rsx! {
        ToastProvider {
            Router::<Route> {}
        }
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let locale = crate::i18n::use_locale();
    rsx! {
        div { class: "not-found",
            h1 { "404" }
            p { {locale.t("error-not-found")} }
            Link { to: Route::SearchPanel {}, {locale.t("action-back-to-search")} }
        }
    }
}
