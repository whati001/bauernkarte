//! store-detail capability: `GET /api/store/{id}`, `GET /api/store/back`,
//! and the full-page `GET /store/{id}` deep link (handled in `pages.rs`,
//! reusing `render_detail_panel` here).

use askama::Template;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
};
use futures_util::stream;
use std::convert::Infallible;

use crate::{
    auth::OptionalUser,
    db,
    dstar::DatastarSignals,
    error::{AppError, AppResult},
    handlers::search::{render_map_data, render_search_panel, run_search, SearchQuery},
    i18n as filters, // see templates.rs's comment on this alias
    models::{RatingCount, StoreDetail},
    sse::{patch_elements, patch_elements_at},
    state::AppState,
    templates::render,
};

struct ImageView {
    id: i64,
    description: Option<String>,
}

struct StoreProductView {
    store_product_id: i64,
    product_id: i64,
    product_name: String,
    product_description: Option<String>,
    product_icon: Option<String>,
    ratings: Vec<RatingCount>,
    viewer_has_rated_up: bool,
    images: Vec<ImageView>,
    selected: bool,
}

#[derive(Template)]
#[template(path = "partials/sidebar_detail.html")]
struct SidebarDetailTemplate {
    store_id: i64,
    store_name: String,
    openinghours: Option<String>,
    lat: f64,
    lon: f64,
    company_id: i64,
    company_name: String,
    company_description: Option<String>,
    company_homepage: Option<String>,
    products: Vec<StoreProductView>,
    logged_in: bool,
}

pub fn render_detail_panel(detail: &StoreDetail, logged_in: bool) -> String {
    render_detail_panel_with_selection(detail, logged_in, None)
}

pub fn render_detail_panel_with_selection(
    detail: &StoreDetail,
    logged_in: bool,
    selected_store_product_id: Option<i64>,
) -> String {
    let products = detail
        .products
        .iter()
        .map(|p| StoreProductView {
            store_product_id: p.store_product_id,
            product_id: p.product_id,
            product_name: p.product_name.clone(),
            product_description: p.product_description.clone(),
            product_icon: p.product_icon.clone(),
            ratings: p.ratings.clone(),
            viewer_has_rated_up: p.viewer_has_rated_up,
            images: p
                .images
                .iter()
                .map(|i| ImageView { id: i.id, description: i.description.clone() })
                .collect(),
            selected: selected_store_product_id == Some(p.store_product_id),
        })
        .collect();

    render(SidebarDetailTemplate {
        store_id: detail.store_id,
        store_name: detail.store_name.clone(),
        openinghours: detail.openinghours.clone(),
        lat: detail.lat,
        lon: detail.lon,
        company_id: detail.company_id,
        company_name: detail.company_name.clone(),
        company_description: detail.company_description.clone(),
        company_homepage: detail.company_homepage.clone(),
        products,
        logged_in,
    })
}

pub async fn load_detail_or_404(state: &AppState, store_id: i64, viewer_id: Option<i64>) -> AppResult<StoreDetail> {
    db::detail::get_store_detail(&state.pool, store_id, viewer_id)
        .await?
        .ok_or(AppError::NotFound)
}

/// `GET /api/store/{id}` — `patch-elements #sidebar` (mode `inner`) with
/// the detail fragment (store-detail capability).
pub async fn show(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    OptionalUser(user): OptionalUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let viewer_id = user.as_ref().map(|u| u.id);
    let detail = load_detail_or_404(&state, store_id, viewer_id).await?;
    tracing::debug!(store_id = %store_id, viewer_id = ?viewer_id, "store detail viewed");
    let html = render_detail_panel(&detail, user.is_some());
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `GET /api/store/back` — re-renders the search panel using whatever
/// filter/location signals the client currently holds (Datastar sends
/// all current signals as query params on a GET action by default, so
/// this reflects the exact prior search without the server tracking any
/// "previous state" itself).
pub async fn back(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let results = run_search(&state, &q).await?;
    let panel = render_search_panel(&state, q.category_id, q.product_id, &results).await?;
    let map_data_html = render_map_data(&panel.map_stores);
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &panel.sidebar_html)),
        Ok(patch_elements(&map_data_html)),
    ])))
}
