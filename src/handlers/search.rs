//! store-search capability: `GET /api/stores`, `GET /api/filters/categories`,
//! `GET /api/filters/products`. Also the shared fragment-building helpers
//! `pages.rs` reuses for the initial full-page render (`GET /`).

use askama::Template;
use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures_util::stream;
use serde::Deserialize;
use std::convert::Infallible;

use crate::{
    db,
    dstar::DatastarSignals,
    error::AppResult,
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    models::{Category, MapStorePin, Product, StoreSearchResult},
    sse::{patch_elements, patch_signals},
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
    top_product_name: Option<String>,
    top_product_rating_count: Option<i64>,
    distance_km_display: String,
}

impl From<&StoreSearchResult> for StoreResultView {
    fn from(r: &StoreSearchResult) -> Self {
        StoreResultView {
            id: r.id,
            name: r.name.clone(),
            top_product_name: r.top_product_name.clone(),
            top_product_rating_count: r.top_product_rating_count,
            distance_km_display: format!("{:.1}", r.distance_m / 1000.0),
        }
    }
}

#[derive(Template)]
#[template(path = "partials/results_list.html")]
struct ResultsListTemplate {
    results: Vec<StoreResultView>,
    result_count_text: String,
    /// Every approved store matching the category/product filter
    /// *nationwide* — feeds the map's pins (`map.js` reads
    /// `#map-stores-json`) and a parallel hidden click-trigger per store,
    /// deliberately not narrowed by "Umkreis": that radius only limits
    /// `results`/`result_count_text` above.
    map_stores: Vec<MapStorePin>,
    map_stores_json: String,
}

#[derive(Template)]
#[template(path = "partials/sidebar_search.html")]
struct SidebarSearchTemplate {
    categories: Vec<Category>,
    products: Vec<Product>,
    results_html: String,
}

pub fn render_results(results: &[StoreSearchResult], map_stores: &[MapStorePin]) -> String {
    let map_stores_json = serde_json::to_string(map_stores).unwrap_or_else(|_| "[]".to_string());
    let mut args = std::collections::HashMap::new();
    args.insert("count".to_string(), results.len().to_string());
    let result_count_text =
        i18n::translate_with_args(i18n::current_locale(), "search-results-count", &args);
    render(ResultsListTemplate {
        results: results.iter().map(StoreResultView::from).collect(),
        result_count_text,
        map_stores: map_stores.to_vec(),
        map_stores_json,
    })
}

/// Builds the full search panel (filters + an initial results list) —
/// used both by `GET /api/store/back` and by `pages::index` for the
/// server-rendered first paint.
pub async fn render_search_panel(
    state: &AppState,
    category_id: Option<i64>,
    product_id: Option<i64>,
    results: &[StoreSearchResult],
) -> AppResult<String> {
    let categories = db::category::list_all(&state.pool).await?;
    let products = match category_id {
        Some(cid) => db::product::list_approved_by_category(&state.pool, cid).await?,
        None => Vec::new(),
    };
    let map_stores = db::store::search_all_for_map(&state.pool, product_id, category_id).await?;
    Ok(render(SidebarSearchTemplate {
        categories,
        products,
        results_html: render_results(results, &map_stores),
    }))
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
    let html = render_results(&results, &map_stores);

    let events = vec![
        patch_elements(&html),
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
        None => Vec::new(),
    };
    let html = render(ProductOptionsTemplate { products });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements(&html))])))
}
