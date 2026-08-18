//! Full-page GET handlers: `GET /`, `GET /store/{id}`, `GET /offline`.

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use serde_json::json;

use crate::{
    auth::OptionalUser,
    db,
    error::AppResult,
    handlers::search::{render_map_data, render_search_panel, run_search, SearchQuery, AUSTRIA_LAT, AUSTRIA_LON},
    handlers::store_detail::{load_detail_or_404, render_detail_panel},
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    state::AppState,
    templates::full_page,
};

/// How many products the navbar's quick-pick row offers. Small on
/// purpose: the row is a shortcut to the handful of things people
/// actually come looking for, not a second catalog browser — the search
/// box and the sidebar's `<select>`s reach everything else. The row
/// scrolls horizontally rather than wrapping, so this is about how far
/// someone should have to scroll, not about fitting a fixed width.
pub const NAV_PRODUCT_LIMIT: i64 = 12;

/// `geo_available: None` renders `$geoAvailable` as JSON `null` —
/// "not yet determined" — distinct from `Some(false)` ("checked, denied/
/// unsupported"). Only `Some(true)` makes the results distance-ranked
/// (`SearchQuery::origin`); the other two both mean `lat`/`lon` hold the
/// Austria-centroid fallback, which is fine for centring the map but is
/// not the visitor's position, so the list stays alphabetical.
pub(crate) fn base_signals(lat: f64, lon: f64, geo_available: Option<bool>, logged_in: bool) -> serde_json::Value {
    json!({
        "categoryId": "", "productId": "",
        "lat": lat, "lon": lon, "geoAvailable": geo_available,
        "resultCount": 0, "selectedStoreId": null, "loggedIn": logged_in,
        // Navbar global search: the picked category/product's name and
        // emoji (both shown in the box) and whether its suggestion
        // dropdown is open. These live in the page-wide signal set
        // because the navbar outlives every #sidebar swap. The quick-pick
        // row needs no signal of its own — it highlights off `$productId`.
        "navQuery": "", "navIcon": "", "navOpen": false,
        // The collapsed navbar's dropdown (navbar.html's .nav-menu).
        // Page-wide for the same reason as the three above: #navbar
        // outlives every #sidebar swap, and is itself re-rendered whole
        // on login/logout — a signal is what survives both.
        "navMenuOpen": false,
    })
}

/// `GET /` — full page, map shell + default search sidebar, per the
/// store-search capability. Geolocation status is unknown at first
/// paint, so this renders every matching store in alphabetical order;
/// the client overrides `$lat/$lon/$geoAvailable` once the browser
/// resolves a fix (`static/map.js`), which re-triggers `/api/stores`
/// through `data-effect` and re-ranks the same list by distance.
pub async fn index(State(state): State<AppState>, OptionalUser(user): OptionalUser) -> AppResult<impl IntoResponse> {
    let q = SearchQuery {
        category_id: None,
        product_id: None,
        lat: Some(AUSTRIA_LAT),
        lon: Some(AUSTRIA_LON),
        // First paint has no fix yet, so results come back alphabetical;
        // map.js re-runs the search through `data-effect` once
        // geolocation resolves, which is when the distance ranking (and
        // the per-card distance) appears.
        geo_available: None,
    };
    let results = run_search(&state, &q).await?;
    let sidebar_html = render_search_panel(&state, None, &results).await?;
    let map_data_html = render_map_data(&results);
    let signals = base_signals(AUSTRIA_LAT, AUSTRIA_LON, None, user.is_some());
    let nav_products = db::product::list_top_rated(&state.pool, NAV_PRODUCT_LIMIT).await?;
    Ok(full_page(
        "BauernKarte",
        user.as_ref(),
        &signals,
        sidebar_html,
        map_data_html,
        // Map-first: the search panel is rendered but starts collapsed —
        // it's one click away (the map's stack button), and the navbar's
        // global search covers the common "filter to one product" case
        // without opening it at all.
        true,
        nav_products,
    ))
}

/// `GET /store/{id}` — full page, sidebar pre-loaded to that store's
/// detail (deep link / share URL), per the store-detail capability.
pub async fn store_page(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    OptionalUser(user): OptionalUser,
) -> AppResult<axum::response::Response> {
    let viewer_id = user.as_ref().map(|u| u.id);
    let detail = load_detail_or_404(&state, store_id, viewer_id).await?;
    let title = format!("{} – BauernKarte", detail.store_name);
    let sidebar_html = render_detail_panel(&detail, user.is_some());
    // Unfiltered, matching the signal defaults below ($categoryId/
    // $productId both "") — a store detail deep link carries no search
    // filter of its own, but the map behind it (see full_page's own doc
    // comment) still needs *some* pin set on first paint, same as the
    // plain search landing page gets.
    let map_stores = db::store::search(&state.pool, None, None, None).await?;
    let map_data_html = render_map_data(&map_stores);
    // Geolocation status genuinely doesn't matter for a detail deep
    // link, but the signal set is shared app-wide (the visitor might hit
    // "Zurück" into search from here), so it still needs a sensible
    // starting point.
    let signals = base_signals(detail.lat, detail.lon, None, user.is_some());
    let nav_products = db::product::list_top_rated(&state.pool, NAV_PRODUCT_LIMIT).await?;
    Ok(
        full_page(&title, user.as_ref(), &signals, sidebar_html, map_data_html, false, nav_products)
            .into_response(),
    )
}

pub async fn healthz() -> Html<&'static str> {
    Html("ok")
}

/// `GET /offline` — the page `static/sw.js` precaches and serves when a
/// navigation can't reach the network.
///
/// Server-rendered rather than a static file in `static/` so its copy
/// goes through Fluent like the rest of the app; the worker re-fetches
/// it after each successful navigation, which is what carries a language
/// switch through to the cached copy. It renders its own document
/// (`templates/offline.html`), not `full_page` — the shell would drag in
/// Leaflet, Datastar and map.js, all of which need the network this page
/// exists to apologise for.
pub async fn offline() -> Html<String> {
    Html(crate::templates::render(OfflineTemplate {
        current_locale: i18n::current_locale().code(),
    }))
}

#[derive(askama::Template)]
#[template(path = "offline.html")]
struct OfflineTemplate {
    current_locale: &'static str,
}
