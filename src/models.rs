//! Row structs mirroring the schema (migrations/). One `FromRow` struct per
//! table plus the handful of query-specific DTOs that don't map 1:1 onto a
//! table (search results, detail views).

use serde::Serialize;
use sqlx::types::Json;
// `time::OffsetDateTime`, not `chrono` — `tower-sessions-sqlx-store`
// already forces sqlx's `time` feature on for its own session-expiry
// columns; enabling `chrono` too on the same unified sqlx build made
// `query_as!`'s TIMESTAMPTZ decoding ambiguous between the two crates
// (discovered while wiring migrations — the macro tried to convert
// between them, and there's no `From` impl). One temporal crate, not two.
use time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub pwd_hash: String,
    pub verified: bool,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    /// A plain-text emoji, not one of the vendored SVG icons — native
    /// `<option>` elements can't render markup, only text (see the
    /// `category_icon` migration).
    pub icon: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Product {
    pub id: i64,
    pub category: i64,
    pub name: String,
    pub description: Option<String>,
    /// A plain-text emoji, same treatment (and same rationale — native
    /// `<option>` elements can't render markup) as `Category::icon` (see
    /// the `product_icon` migration).
    pub icon: Option<String>,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

/// One weekday's opening hours (`store.openinghours`, stored as a sparse
/// JSONB array — a day with no entry is closed). `day` is ISO 8601
/// weekday numbering (1 = Monday .. 7 = Sunday). Expanded into a fixed
/// 7-row week for display/editing by `opening_hours::week_rows`.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DayHours {
    pub day: i16,
    pub open: String,
    pub close: String,
}

/// `store.position` (a PostGIS `geography(Point,4326)`) has no sqlx
/// scalar mapping, so every store query projects it as `lat`/`lon` via
/// `ST_Y`/`ST_X` in SQL rather than selecting the geography column
/// directly (see db/store.rs). `openinghours` stays `sqlx::types::Json`
/// here (not unwrapped to a plain `Vec`) since `Store` is never handed
/// to an Askama template directly — only the assembled `StoreDetail`/
/// form-view structs are, and those carry the unwrapped `Vec<DayHours>`.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Store {
    pub id: i64,
    pub company: i64,
    pub name: String,
    pub openinghours: Option<Json<Vec<DayHours>>>,
    pub lat: f64,
    pub lon: f64,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

/// `seasonal_months`: `None` = available all year (the default/common
/// case), `Some(months)` = only those months (1 = January .. 12 =
/// December) — see `seasonality::parse`. Same "stays `Json` here, only
/// the assembled detail struct unwraps it" reasoning as `Store::openinghours`.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct StoreProduct {
    pub id: i64,
    pub store: i64,
    pub product: i64,
    pub seasonal_months: Option<Json<Vec<i16>>>,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Image {
    pub id: i64,
    pub store_product: i64,
    #[serde(skip)]
    pub image: Vec<u8>,
    pub mime_type: String,
    pub description: Option<String>,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

/// One product a store carries, as summarized for search/map results —
/// name + icon (for display) and its `UP`-rating count (for ranking).
/// Never fetched standalone; always one entry of the top-5 list
/// `db::store::search` computes in SQL (`json_agg` over a ranked,
/// `limit 5` subquery) and attaches to `StoreSearchResult`,
/// ranked by `rating_count` descending, ties broken
/// alphabetically by `name` — one query per search, no per-store
/// follow-up request (store-search capability's "single search request"
/// requirement).
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ProductSummary {
    pub name: String,
    pub icon: Option<String>,
    pub rating_count: i64,
}

/// One matching store — the same rows drive both the results list and
/// the map's pins (see `db::store::search`; there is no radius, so the
/// two are the same set).
///
/// `distance_m` is `None` until geolocation resolves: with no real
/// origin there is no distance worth showing, and the list falls back to
/// alphabetical order.
#[derive(Debug, Clone, Serialize)]
pub struct StoreSearchResult {
    pub id: i64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub distance_m: Option<f64>,
    /// Top 5 by rating, see `ProductSummary`. The store may carry more —
    /// `product_total` says how many, so the UI can show a "+N more"
    /// indicator instead of silently truncating.
    pub products: Vec<ProductSummary>,
    pub product_total: i64,
}

/// One entry of the navbar's quick-pick product row. The ranking (total
/// ratings across *every* store carrying the product, ties broken
/// alphabetically) lives entirely in `db::product::list_top_rated`'s
/// `order by` — the count itself is never shown, so it isn't carried
/// here. Just enough to render `icon + name` and know what to filter by.
#[derive(Debug, Clone)]
pub struct RankedProduct {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
}

/// A count of ratings of one `rating_type` on a `store_product`, generic
/// over the type so the template can render `<count> <icon>` per type
/// without a rewrite when a second type is added (ratings capability).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct RatingCount {
    pub rating_type_id: i64,
    pub rating_type_name: String,
    pub count: i64,
}

/// One product row as shown on a store's detail page: the store_product
/// listing joined with its product, plus rating counts and whether the
/// current viewer has already rated it.
#[derive(Debug, Clone)]
pub struct StoreProductDetail {
    pub store_product_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub product_description: Option<String>,
    pub product_icon: Option<String>,
    /// `None` = available all year, see `StoreProduct::seasonal_months`.
    pub seasonal_months: Option<Vec<i16>>,
    pub ratings: Vec<RatingCount>,
    pub viewer_has_rated_up: bool,
    pub images: Vec<ImageSummary>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ImageSummary {
    pub id: i64,
    pub description: Option<String>,
}

/// Full store detail view (store-detail capability).
#[derive(Debug, Clone)]
pub struct StoreDetail {
    pub store_id: i64,
    pub store_name: String,
    /// Sparse — a day absent from the list is closed. Empty means "not
    /// specified at all" (the detail view hides the section entirely
    /// then, same as before this was structured — see
    /// `opening_hours::week_rows`).
    pub openinghours: Vec<DayHours>,
    pub lat: f64,
    pub lon: f64,
    pub company_id: i64,
    pub company_name: String,
    pub company_description: Option<String>,
    pub company_homepage: Option<String>,
    pub products: Vec<StoreProductDetail>,
}
