//! store-search capability: `GET /api/stores`, `GET /api/filters/categories`,
//! `GET /api/filters/products`, and the navbar's global search box
//! (`GET /api/search/suggest`, `GET /api/search/select/{kind}/{id}`). Also
//! the shared fragment-building helpers `pages.rs` reuses for the initial
//! full-page render (`GET /`).

use askama::Template;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
};
use futures_util::stream;
use serde::Deserialize;
use std::convert::Infallible;

use crate::{
    db,
    dstar::DatastarSignals,
    error::{AppError, AppResult},
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    models::{Category, MapStorePin, Product, ProductSummary, StoreSearchResult},
    sse::{patch_elements, patch_elements_at, patch_signals},
    state::AppState,
    templates::render,
};

/// Austria's geographic centroid — the no-geolocation-permission fallback
/// (design.md §8.1), not Vienna, so the default view isn't biased toward
/// the capital.
pub const AUSTRIA_LAT: f64 = 47.5162;
pub const AUSTRIA_LON: f64 = 14.5501;
pub const DEFAULT_DISTANCE_KM: f64 = 5.0;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub category_id: Option<i64>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub product_id: Option<i64>,
    pub distance_km: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

/// `<select>`/`<input>` elements bound via `data-bind` send `""` (not
/// absent) for an unselected/empty value, and that JSON `""` won't
/// deserialize into `Option<i64>` — treat it the same as `null`/absent.
fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
        Null,
    }
    match Option::<StringOrInt>::deserialize(deserializer)? {
        None | Some(StringOrInt::Null) => Ok(None),
        Some(StringOrInt::String(s)) if s.is_empty() => Ok(None),
        Some(StringOrInt::String(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
    }
}

struct StoreResultView {
    id: i64,
    name: String,
    /// Top 5 by rating (see `ProductSummary`); `products_more` is how
    /// many beyond those 5 the store also carries, for a "+N more"
    /// affordance instead of silently truncating the list.
    products: Vec<ProductSummary>,
    products_more: i64,
    distance_km_display: String,
}

impl From<&StoreSearchResult> for StoreResultView {
    fn from(r: &StoreSearchResult) -> Self {
        StoreResultView {
            id: r.id,
            name: r.name.clone(),
            products: r.products.clone(),
            products_more: (r.product_total - r.products.len() as i64).max(0),
            distance_km_display: format!("{:.1}", r.distance_m / 1000.0),
        }
    }
}

#[derive(Template)]
#[template(path = "partials/results_list.html")]
struct ResultsListTemplate {
    results: Vec<StoreResultView>,
    result_count_text: String,
    /// Pre-translated "no results in your radius, but N exist further
    /// away" message — `Some` only when `results` is empty *and* at
    /// least one nationwide match exists, so an empty "Umkreis" search
    /// doesn't read as a dead end when the map still shows real pins
    /// just outside it (e.g. a product only sold by stores >100 km out —
    /// beyond even the radius slider's own max). `None` (including
    /// whenever `results` isn't empty) falls back to the template's
    /// plain `search-no-results`.
    nationwide_empty_message: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/sidebar_search.html")]
struct SidebarSearchTemplate {
    categories: Vec<Category>,
    products: Vec<Product>,
    results_html: String,
}

/// The map's pin data (`#map-stores-json` + one hidden click-trigger per
/// store) — deliberately its own fragment, patched to a container that
/// lives *outside* `#sidebar` (`layout.html`'s `#map-data`) rather than
/// nested inside the search results. Nesting it there used to mean the
/// map's entire pin set (and the triggers `map.js` proxy-clicks to open
/// a store) vanished the instant the sidebar swapped to a detail panel
/// or form — silently breaking "select a different store while one is
/// already open" (no trigger to click) and "the pins survive a zoom"
/// (redrawMarkers saw no `#map-stores-json` and wiped everything). A
/// stable, always-present container fixes both: which store's detail
/// panel `#sidebar` currently shows no longer has anything to do with
/// whether the map's pins exist.
#[derive(Template)]
#[template(path = "partials/map_data.html")]
struct MapDataTemplate {
    map_stores: Vec<MapStorePin>,
    map_stores_json: String,
}

pub fn render_map_data(map_stores: &[MapStorePin]) -> String {
    let map_stores_json = serde_json::to_string(map_stores).unwrap_or_else(|_| "[]".to_string());
    render(MapDataTemplate { map_stores: map_stores.to_vec(), map_stores_json })
}

pub fn render_results(results: &[StoreSearchResult], nationwide_count: usize) -> String {
    let mut args = std::collections::HashMap::new();
    args.insert("count".to_string(), results.len().to_string());
    let result_count_text =
        i18n::translate_with_args(i18n::current_locale(), "search-results-count", &args);
    let nationwide_empty_message = (results.is_empty() && nationwide_count > 0).then(|| {
        let mut args = std::collections::HashMap::new();
        args.insert("count".to_string(), nationwide_count.to_string());
        i18n::translate_with_args(i18n::current_locale(), "search-no-results-nearby", &args)
    });
    render(ResultsListTemplate {
        results: results.iter().map(StoreResultView::from).collect(),
        result_count_text,
        nationwide_empty_message,
    })
}

/// Sidebar search panel (filters + an initial results list) plus the
/// nationwide store set the caller needs for the *separate* `#map-data`
/// patch — bundled together, not two independent queries, since both are
/// derived from the same `search_all_for_map` call.
pub struct SearchPanel {
    pub sidebar_html: String,
    pub map_stores: Vec<MapStorePin>,
}

/// Builds `SearchPanel` — used by `GET /api/store/back` and by
/// `pages::index` for the server-rendered first paint, plus every
/// catalog-editing route that returns to search after a delete.
pub async fn render_search_panel(
    state: &AppState,
    category_id: Option<i64>,
    product_id: Option<i64>,
    results: &[StoreSearchResult],
) -> AppResult<SearchPanel> {
    let categories = db::category::list_all(&state.pool).await?;
    let products = match category_id {
        Some(cid) => db::product::list_approved_by_category(&state.pool, cid).await?,
        None => db::product::list_all_approved(&state.pool).await?,
    };
    let map_stores = db::store::search_all_for_map(&state.pool, product_id, category_id).await?;
    let sidebar_html = render(SidebarSearchTemplate {
        categories,
        products,
        results_html: render_results(results, map_stores.len()),
    });
    Ok(SearchPanel { sidebar_html, map_stores })
}

pub async fn run_search(state: &AppState, q: &SearchQuery) -> AppResult<Vec<StoreSearchResult>> {
    let lat = q.lat.unwrap_or(AUSTRIA_LAT);
    let lon = q.lon.unwrap_or(AUSTRIA_LON);
    let distance_km = q.distance_km.unwrap_or(DEFAULT_DISTANCE_KM);
    let results = db::store::search(&state.pool, lat, lon, distance_km, q.product_id, q.category_id).await?;
    Ok(results)
}

/// `GET /api/stores` — Datastar SSE: `patch-signals {resultCount}` +
/// `patch-elements #sidebar-results`. (design.md route table)
pub async fn stores(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let results = run_search(&state, &q).await?;
    let count = results.len();
    let map_stores = db::store::search_all_for_map(&state.pool, q.product_id, q.category_id).await?;
    let results_html = render_results(&results, map_stores.len());
    let map_data_html = render_map_data(&map_stores);
    tracing::debug!(
        category_id = ?q.category_id, product_id = ?q.product_id, distance_km = ?q.distance_km,
        result_count = count, nationwide_count = map_stores.len(),
        "search executed"
    );

    let events = vec![
        patch_elements(&results_html),
        patch_elements(&map_data_html),
        patch_signals(&serde_json::json!({ "resultCount": count })),
    ];
    Ok(Sse::new(stream::iter(events.into_iter().map(Ok))))
}

#[derive(Template)]
#[template(path = "partials/category_options.html")]
struct CategoryOptionsTemplate {
    categories: Vec<Category>,
}

/// `GET /api/filters/categories` — `patch-elements` for the `<select>`
/// options (design.md route table). In practice the select is rendered
/// server-side up front (categories are a fixed, small taxonomy) so this
/// route exists mainly for symmetry / future re-seeding without a reload;
/// it's wired but not polled by the default UI flow.
pub async fn filter_categories(
    State(state): State<AppState>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let categories = db::category::list_all(&state.pool).await?;
    let html = render(CategoryOptionsTemplate { categories });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements(&html))])))
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryFilterQuery {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub category_id: Option<i64>,
}

#[derive(Template)]
#[template(path = "partials/product_options.html")]
struct ProductOptionsTemplate {
    products: Vec<Product>,
}

/// `GET /api/filters/products?category_id=` — cascading product options
/// for the selected category (store-search capability).
pub async fn filter_products(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<CategoryFilterQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let products = match q.category_id {
        Some(cid) => db::product::list_approved_by_category(&state.pool, cid).await?,
        None => db::product::list_all_approved(&state.pool).await?,
    };
    let html = render(ProductOptionsTemplate { products });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements(&html))])))
}

/// How many of each kind the navbar dropdown offers. Deliberately small:
/// the box exists to pick a known category/product, not to browse the
/// whole catalog — that's what the sidebar's `<select>`s are for.
const SUGGEST_LIMIT: i64 = 6;

/// One row of the navbar's suggestion dropdown. Categories and products
/// share one flat list (one dropdown, not two), so the kind has to travel
/// with each row: `kind` doubles as the `/api/search/select/{kind}/{id}`
/// path segment, `kind_label` is its translated badge.
pub struct Suggestion {
    pub kind: &'static str,
    pub id: i64,
    /// Already defaulted to the same 🏷️/📦 fallbacks the filter
    /// `<option>`s use, so the template stays free of that branch.
    pub icon: String,
    pub name: String,
    pub kind_label: String,
}

#[derive(Template)]
#[template(path = "partials/nav_suggestions.html")]
pub struct NavSuggestionsTemplate {
    pub suggestions: Vec<Suggestion>,
    /// Distinguishes "typed something, nothing matched" (show the
    /// no-matches line) from "box is empty" (show nothing at all) —
    /// `suggestions.is_empty()` alone can't tell those apart.
    pub searched: bool,
}

/// The empty dropdown the navbar ships with on a full-page render — the
/// `#nav-suggestions` element has to exist in the DOM before `suggest`
/// can patch it by id.
pub fn render_empty_suggestions() -> String {
    render(NavSuggestionsTemplate { suggestions: Vec::new(), searched: false })
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestQuery {
    #[serde(default)]
    pub nav_query: String,
}

/// `GET /api/search/suggest` — the navbar box is a *picker*, not a
/// free-text search: this only ever returns existing categories/products,
/// and nothing happens until one of them is clicked (`select` below). The
/// `navOpen` signal is set here rather than client-side so an emptied box
/// closes the dropdown without a second round trip.
pub async fn suggest(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SuggestQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let term = q.nav_query.trim();
    let locale = i18n::current_locale();
    let mut suggestions = Vec::new();

    if !term.is_empty() {
        let category_label = i18n::translate(locale, "search-category");
        let product_label = i18n::translate(locale, "search-product");
        for c in db::category::search_by_name(&state.pool, term, SUGGEST_LIMIT).await? {
            suggestions.push(Suggestion {
                kind: "category",
                id: c.id,
                icon: c.icon.unwrap_or_else(|| "🏷️".to_string()),
                name: c.name,
                kind_label: category_label.clone(),
            });
        }
        for p in db::product::search_approved_by_name(&state.pool, term, SUGGEST_LIMIT).await? {
            suggestions.push(Suggestion {
                kind: "product",
                id: p.id,
                icon: p.icon.unwrap_or_else(|| "📦".to_string()),
                name: p.name,
                kind_label: product_label.clone(),
            });
        }
    }

    let open = !term.is_empty();
    let html = render(NavSuggestionsTemplate { suggestions, searched: open });
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements(&html)),
        Ok(patch_signals(&serde_json::json!({ "navOpen": open }))),
    ])))
}

/// `$categoryId`/`$productId` as the bound `<select>`s themselves write
/// them: the option's string value, `""` for "Alle" (see
/// `empty_string_as_none`, which reads both back).
fn signal_id(id: Option<i64>) -> String {
    id.map(|v| v.to_string()).unwrap_or_default()
}

/// `GET /api/search/select/{kind}/{id}` — commits one suggestion as the
/// active filter. Server-side rather than a signal assignment inline in
/// the dropdown's `data-on:click` because the box has to end up showing
/// the picked *name*, and interpolating a database string into a
/// JS expression attribute is a quoting bug waiting to happen — here the
/// name only ever travels as JSON in a `patch-signals` payload.
///
/// The whole search panel is re-rendered (not just the results list) so
/// this works identically from a store detail or a form, which is what
/// makes the box "global": wherever you are, picking a suggestion lands
/// you back on search with that filter applied.
pub async fn select(
    State(state): State<AppState>,
    Path((kind, id)): Path<(String, i64)>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    // A product carries its category along, so the sidebar's cascading
    // `<select>`s stay coherent (category "Obst" + product "Äpfel", not
    // a product hanging under "Alle"). Filtering by both is redundant but
    // harmless — the product already implies its category.
    let (name, category_id, product_id) = match kind.as_str() {
        "category" => {
            let c = db::category::find(&state.pool, id).await?.ok_or(AppError::NotFound)?;
            (c.name, Some(c.id), None)
        }
        "product" => {
            let p = db::product::find_approved(&state.pool, id).await?.ok_or(AppError::NotFound)?;
            (p.name, Some(p.category), Some(p.id))
        }
        _ => return Err(AppError::NotFound),
    };

    let q = SearchQuery { category_id, product_id, ..q };
    let results = run_search(&state, &q).await?;
    let panel = render_search_panel(&state, category_id, product_id, &results).await?;
    let map_data_html = render_map_data(&panel.map_stores);
    tracing::debug!(kind = %kind, id = %id, result_count = results.len(), "navbar suggestion selected");

    // Elements *before* signals, and the order genuinely matters: the
    // product `<select>`'s options are rebuilt by this patch (they cascade
    // from the new category), and a `<select>` silently drops its value
    // when the matching `<option>` is replaced. Patching the signals
    // first would set the value on the old option list and then lose it
    // to the morph; this way `data-bind:product-id` re-applies
    // `$productId` against the options that are actually there.
    //
    // `categoryId`/`productId` go over as `<select>`-shaped strings
    // (`""` for "Alle") — never JSON `null`, which in a `patch-signals`
    // payload *removes* the signal rather than blanking it (see sse.rs).
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements(&render_empty_suggestions())),
        Ok(patch_elements_at("#sidebar", "inner", &panel.sidebar_html)),
        Ok(patch_elements(&map_data_html)),
        Ok(patch_signals(&serde_json::json!({
            "categoryId": signal_id(category_id),
            "productId": signal_id(product_id),
            "navQuery": name,
            "navOpen": false,
            "resultCount": results.len(),
        }))),
    ])))
}
