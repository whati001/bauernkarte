//! Row structs mirroring the schema (migrations/). One `FromRow` struct per
//! table plus the handful of query-specific DTOs that don't map 1:1 onto a
//! table (search results, detail views).

use rust_decimal::Decimal;
use serde::Serialize;
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
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

/// `store.position` (a PostGIS `geography(Point,4326)`) has no sqlx
/// scalar mapping, so every store query projects it as `lat`/`lon` via
/// `ST_Y`/`ST_X` in SQL rather than selecting the geography column
/// directly (see db/store.rs).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Store {
    pub id: i64,
    pub company: i64,
    pub name: String,
    pub openinghours: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
    pub modified_by: Option<i64>,
    pub created: OffsetDateTime,
    pub modified: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct StoreProduct {
    pub id: i64,
    pub store: i64,
    pub product: i64,
    pub price: Decimal,
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

/// One row of a store-search result — the "Umkreis" (radius) filtered
/// list (store-search capability — kept minimal per design.md: id, name,
/// coordinates, distance, best product, its rating count).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct StoreSearchResult {
    pub id: i64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub distance_m: f64,
    pub top_product_name: Option<String>,
    pub top_product_rating_count: Option<i64>,
}

/// One map pin — every approved store matching the category/product
/// filter *nationwide*, deliberately not distance-limited. The map shows
/// every pin regardless of the "Umkreis" radius (only the results list
/// is radius-filtered, per the follow-up that split this from
/// `StoreSearchResult`); no `distance_m` since it isn't measured against
/// any particular origin.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct MapStorePin {
    pub id: i64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub top_product_name: Option<String>,
    pub top_product_rating_count: Option<i64>,
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
    pub price: Decimal,
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
    pub openinghours: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub company_id: i64,
    pub company_name: String,
    pub company_description: Option<String>,
    pub company_homepage: Option<String>,
    pub products: Vec<StoreProductDetail>,
}
