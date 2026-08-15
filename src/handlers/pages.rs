//! Full-page GET handlers: `GET /`, `GET /store/{id}`.

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use serde_json::json;

use crate::{
    auth::OptionalUser,
    db,
    db::store::MAX_DISTANCE_KM,
    error::AppResult,
    handlers::search::{render_map_data, render_search_panel, run_search, SearchQuery, AUSTRIA_LAT, AUSTRIA_LON},
    handlers::store_detail::{load_detail_or_404, render_detail_panel},
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
/// unsupported"). The search-radius control and its map circle
/// (`sidebar_search.html`, `map.js`) only show once it's `Some(true)`:
/// a radius drawn around the Austria-centroid fallback would imply a
/// precision the app doesn't have. While unresolved-or-unavailable, the
/// search itself still runs, just at the spec's full 100 km cap instead
/// of a arbitrarily-anchored 5 km default.
fn base_signals(lat: f64, lon: f64, geo_available: Option<bool>, logged_in: bool) -> serde_json::Value {
    let distance_km = if geo_available == Some(true) { 5.0 } else { MAX_DISTANCE_KM };
    json!({
        "categoryId": "", "productId": "", "distanceKm": distance_km,
        "lat": lat, "lon": lon, "geoAvailable": geo_available,
        "resultCount": 0, "selectedStoreId": null, "loggedIn": logged_in,
        // Navbar global search: the picked category/product's name and
        // emoji (both shown in the box) and whether its suggestion
        // dropdown is open. These live in the page-wide signal set
        // because the navbar outlives every #sidebar swap. The quick-pick
        // row needs no signal of its own — it highlights off `$productId`.
        "navQuery": "", "navIcon": "", "navOpen": false,
    })
}

/// `GET /` — full page, map shell + default search sidebar, per the
/// store-search capability. Server-side first paint uses the Austria
/// centroid fallback at the full 100 km radius (design.md §8.1, extended
/// per follow-up: geolocation status is unknown at this point, so no
/// radius is assumed); the client overrides `$lat/$lon/$geoAvailable`
/// via geolocation once the browser resolves it (`static/map.js`), which
/// re-triggers `/api/stores` through `data-effect` and — if geolocation
/// succeeded — narrows the default radius back to 5 km.
pub async fn index(State(state): State<AppState>, OptionalUser(user): OptionalUser) -> AppResult<impl IntoResponse> {
    let q = SearchQuery {
        category_id: None,
        product_id: None,
        distance_km: Some(MAX_DISTANCE_KM),
        lat: Some(AUSTRIA_LAT),
        lon: Some(AUSTRIA_LON),
    };
    let results = run_search(&state, &q).await?;
    let panel = render_search_panel(&state, None, None, &results).await?;
    let map_data_html = render_map_data(&panel.map_stores);
    let signals = base_signals(AUSTRIA_LAT, AUSTRIA_LON, None, user.is_some());
    let nav_products = db::product::list_top_rated(&state.pool, NAV_PRODUCT_LIMIT).await?;
    Ok(full_page(
        "Was hat der Bauer",
        user.map(|u| u.name),
        &signals,
        panel.sidebar_html,
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
    let title = format!("{} – Was hat der Bauer", detail.store_name);
    let sidebar_html = render_detail_panel(&detail, user.is_some());
    // Unfiltered, matching the signal defaults below ($categoryId/
    // $productId both "") — a store detail deep link carries no search
    // filter of its own, but the map behind it (see full_page's own doc
    // comment) still needs *some* pin set on first paint, same as the
    // plain search landing page gets.
    let map_stores = db::store::search_all_for_map(&state.pool, None, None).await?;
    let map_data_html = render_map_data(&map_stores);
    // Geolocation status genuinely doesn't matter for a detail deep
    // link, but the signal set is shared app-wide (the visitor might hit
    // "Zurück" into search from here), so it still needs a sensible
    // starting point.
    let signals = base_signals(detail.lat, detail.lon, None, user.is_some());
    let nav_products = db::product::list_top_rated(&state.pool, NAV_PRODUCT_LIMIT).await?;
    Ok(
        full_page(&title, user.map(|u| u.name), &signals, sidebar_html, map_data_html, false, nav_products)
            .into_response(),
    )
}

pub async fn healthz() -> Html<&'static str> {
    Html("ok")
}
