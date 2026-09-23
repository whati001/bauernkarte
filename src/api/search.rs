//! Store search and the product catalog lookups behind the navbar and the
//! filter.

use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::{CatalogProduct, StoreSearchResult};

#[cfg(feature = "server")]
use crate::server::{db, pool};

/// How many products the navbar's quick-pick row offers: a shortcut to
/// the handful people come for, not a second catalog browser.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub const NAV_PRODUCT_LIMIT: i64 = 12;

/// Every approved store carrying `product_id` (or any product). Ranked by
/// distance from `lat`/`lon` only when both are a real fix — the Austria
/// fallback would be a meaningless order dressed up as a meaningful one,
/// so without one the list is alphabetical.
#[get("/api/stores?product_id&lat&lon")]
pub async fn search_stores(
    product_id: Option<i64>,
    lat: Option<f64>,
    lon: Option<f64>,
) -> ApiResult<Vec<StoreSearchResult>> {
    let origin = lat.zip(lon);
    let results = db::store::search(pool(), origin, product_id).await?;
    tracing::debug!(?product_id, ranked_by_distance = origin.is_some(), count = results.len(), "search");
    Ok(results)
}

#[get("/api/products")]
pub async fn all_products() -> ApiResult<Vec<CatalogProduct>> {
    Ok(db::product::list_all_approved(pool()).await?)
}

/// The navbar's quick-pick row: most-rated first.
#[get("/api/products/top")]
pub async fn top_products() -> ApiResult<Vec<CatalogProduct>> {
    Ok(db::product::list_top_rated(pool(), NAV_PRODUCT_LIMIT).await?)
}
