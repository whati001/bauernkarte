//! store-search capability: `GET /api/stores` and the navbar's global
//! search box (`GET /api/search/suggest`,
//! `GET /api/search/select/{id}`). Also the shared
//! fragment-building helpers `pages.rs` reuses for the initial full-page
//! render (`GET /`).

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
    models::{Product, ProductSummary, StoreSearchResult},
    sse::{patch_elements, patch_elements_at, patch_signals},
    state::AppState,
    templates::render,
};

/// Austria's geographic centroid — the no-geolocation-permission fallback
/// (design.md §8.1), not Vienna, so the default view isn't biased toward
/// the capital.
pub const AUSTRIA_LAT: f64 = 47.5162;
pub const AUSTRIA_LON: f64 = 14.5501;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub product_id: Option<i64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    /// `Some(true)` only once the browser has actually returned a fix —
    /// `None` means "not determined yet" and `Some(false)` "denied or
    /// unsupported". `lat`/`lon` carry the Austria-centroid fallback in
    /// those two cases, which is fine for centring the map but must not
    /// be treated as the visitor's position (see `origin`).
    #[serde(default)]
    pub geo_available: Option<bool>,
}

impl SearchQuery {
    /// The point to rank results by, or `None` when there's no real fix
    /// to rank against.
    fn origin(&self) -> Option<(f64, f64)> {
        match (self.geo_available, self.lat, self.lon) {
            (Some(true), Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }
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
    /// `None` when there's no geolocation fix — the card then shows no
    /// distance at all rather than one measured from a fallback point.
    distance_km_display: Option<String>,
}

impl From<&StoreSearchResult> for StoreResultView {
    fn from(r: &StoreSearchResult) -> Self {
        StoreResultView {
            id: r.id,
            name: r.name.clone(),
            products: r.products.clone(),
            products_more: (r.product_total - r.products.len() as i64).max(0),
            distance_km_display: r.distance_m.map(|m| format!("{:.1}", m / 1000.0)),
        }
    }
}

#[derive(Template)]
#[template(path = "partials/results_list.html")]
struct ResultsListTemplate {
    results: Vec<StoreResultView>,
    result_count_text: String,
}

#[derive(Template)]
#[template(path = "partials/sidebar_search.html")]
struct SidebarSearchTemplate {
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
    map_stores: Vec<StoreSearchResult>,
    map_stores_json: String,
}

/// Pins for the same rows the results list shows — with the radius gone
/// the two are the same set, so this takes the search results directly
/// rather than re-querying.
pub fn render_map_data(map_stores: &[StoreSearchResult]) -> String {
    let map_stores_json = serde_json::to_string(map_stores).unwrap_or_else(|_| "[]".to_string());
    render(MapDataTemplate { map_stores: map_stores.to_vec(), map_stores_json })
}

pub fn render_results(results: &[StoreSearchResult]) -> String {
    let mut args = std::collections::HashMap::new();
    args.insert("count".to_string(), results.len().to_string());
    let result_count_text =
        i18n::translate_with_args(i18n::current_locale(), "search-results-count", &args);
    render(ResultsListTemplate {
        results: results.iter().map(StoreResultView::from).collect(),
        result_count_text,
    })
}

/// Builds `SearchPanel` — used by `GET /api/store/back` and by
/// `pages::index` for the server-rendered first paint, plus every
/// catalog-editing route that returns to search after a delete.
pub async fn render_search_panel(
    state: &AppState,
    results: &[StoreSearchResult],
) -> AppResult<String> {
    Ok(render(SidebarSearchTemplate {
        products: db::product::list_all_approved(&state.pool).await?,
        results_html: render_results(results),
    }))
}

pub async fn run_search(state: &AppState, q: &SearchQuery) -> AppResult<Vec<StoreSearchResult>> {
    Ok(db::store::search(&state.pool, q.origin(), q.product_id).await?)
}

/// `GET /api/stores` — Datastar SSE: `patch-signals {resultCount}` +
/// `patch-elements #sidebar-results`. (design.md route table)
pub async fn stores(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let results = run_search(&state, &q).await?;
    let count = results.len();
    let results_html = render_results(&results);
    let map_data_html = render_map_data(&results);
    tracing::debug!(
        product_id = ?q.product_id,
        ranked_by_distance = q.origin().is_some(), result_count = count,
        "search executed"
    );

    let events = vec![
        patch_elements(&results_html),
        patch_elements(&map_data_html),
        patch_signals(&serde_json::json!({ "resultCount": count })),
    ];
    Ok(Sse::new(stream::iter(events.into_iter().map(Ok))))
}

/// How many the navbar dropdown offers. Deliberately small: the box
/// exists to pick a known product, not to browse the whole catalog —
/// that's what the sidebar's `<select>` is for.
const SUGGEST_LIMIT: i64 = 6;

/// One row of the navbar's suggestion dropdown.
pub struct Suggestion {
    pub id: i64,
    /// Already defaulted to the same 📦 fallback the filter `<option>`s
    /// use, so the template stays free of that branch.
    pub icon: String,
    pub name: String,
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
/// free-text search: this only ever returns existing products, and
/// nothing happens until one of them is clicked (`select` below). The
/// `navOpen` signal is set here rather than client-side so an emptied box
/// closes the dropdown without a second round trip.
pub async fn suggest(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SuggestQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let term = q.nav_query.trim();
    let mut suggestions = Vec::new();

    if !term.is_empty() {
        for p in db::product::search_approved_by_name(&state.pool, term, SUGGEST_LIMIT).await? {
            suggestions.push(Suggestion {
                id: p.id,
                icon: p.icon.unwrap_or_else(|| "📦".to_string()),
                name: p.name,
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

/// `$productId` as the bound `<select>` itself writes it: the option's
/// string value, `""` for "Alle" (see `empty_string_as_none`, which reads
/// it back).
fn signal_id(id: Option<i64>) -> String {
    id.map(|v| v.to_string()).unwrap_or_default()
}

/// `GET /api/search/select/{id}` — commits one suggestion as the active
/// filter. Server-side rather than a signal assignment inline in the
/// dropdown's `data-on:click` because the box has to end up showing the
/// picked *name*, and interpolating a database string into a JS
/// expression attribute is a quoting bug waiting to happen — here the
/// name only ever travels as JSON in a `patch-signals` payload.
///
/// The whole search panel is re-rendered (not just the results list) so
/// this works identically from a store detail or a form, which is what
/// makes the box "global": wherever you are, picking a suggestion lands
/// you back on search with that filter applied.
pub async fn select(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let p = db::product::find_approved(&state.pool, id).await?.ok_or(AppError::NotFound)?;
    tracing::debug!(product_id = %id, "navbar filter selected");
    let icon = p.icon.unwrap_or_else(|| "📦".to_string());
    apply_filter(state, q, Some(p.id), p.name, icon).await
}

/// `GET /api/search/clear` — back to "everything", i.e. the same reset
/// the sidebar's own "Alle" options produce. Its own route rather than a
/// signal-assignment expression because three controls trigger it (the
/// search box's ✕, an already-selected product chip, and any future
/// reset affordance) and one server-side definition beats three copies
/// of the same five assignments drifting apart.
pub async fn clear(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    tracing::debug!("navbar filter cleared");
    apply_filter(state, q, None, String::new(), String::new()).await
}

/// Shared by `select` and `clear`: put the given product filter into
/// effect everywhere it shows — sidebar panel, map pins, and the navbar's
/// own state (box label + icon, and via `$productId` the highlighted
/// product chip).
///
/// `label`/`icon` travel as JSON in a `patch-signals` payload rather than
/// being interpolated into the dropdown's `data-on:click` expression —
/// a database string inside a JS attribute is a quoting bug waiting to
/// happen.
async fn apply_filter(
    // By value, not `&AppState`: the returned `impl Stream` would
    // otherwise capture the borrow (edition 2024 captures every in-scope
    // lifetime in an opaque type) and outlive it. `AppState` is a single
    // `PgPool`, so the move costs an `Arc` bump.
    state: AppState,
    q: SearchQuery,
    product_id: Option<i64>,
    label: String,
    icon: String,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let q = SearchQuery { product_id, ..q };
    let results = run_search(&state, &q).await?;
    let sidebar_html = render_search_panel(&state, &results).await?;
    let map_data_html = render_map_data(&results);

    // Elements *before* signals, and the order genuinely matters: the
    // product `<select>`'s options are rebuilt by this patch, and a
    // `<select>` silently drops its value when the matching `<option>` is
    // replaced. Patching the signals first would set the value on the old
    // option list and then lose it to the morph; this way
    // `data-bind:product-id` re-applies `$productId` against the options
    // that are actually there.
    //
    // `productId` goes over as a `<select>`-shaped string (`""` for
    // "Alle") — never JSON `null`, which in a `patch-signals` payload
    // *removes* the signal rather than blanking it (see sse.rs).
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements(&render_empty_suggestions())),
        Ok(patch_elements_at("#sidebar", "inner", &sidebar_html)),
        Ok(patch_elements(&map_data_html)),
        Ok(patch_signals(&serde_json::json!({
            "productId": signal_id(product_id),
            "navQuery": label,
            "navIcon": icon,
            "navOpen": false,
            "resultCount": results.len(),
        }))),
    ])))
}
