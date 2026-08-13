use serde_json::json;
use sqlx::{types::Json, PgPool};

use crate::models::{MapStorePin, ProductSummary, Store, StoreSearchResult};

/// Hard cap enforced here regardless of what a client sends (store-search
/// capability, design.md's 100 km decision) — `4.1`.
pub const MAX_DISTANCE_KM: f64 = 100.0;

/// Mirrors `StoreSearchResult` for `query_as!`'s benefit — `products`
/// comes back as a jsonb column decoded via `sqlx::types::Json`, which
/// isn't `Serialize`/template-iterable the way the plain `Vec` on the
/// public model is, so it's unwrapped (`.0`) once here rather than
/// exposing `Json<..>` further up the stack (same pattern as
/// `db::detail`'s private row structs).
struct SearchRow {
    id: i64,
    name: String,
    lat: f64,
    lon: f64,
    distance_m: f64,
    products: Json<Vec<ProductSummary>>,
    product_total: i64,
}

/// Reference distance query from design.md, filtering `approved and not
/// deleted` at every level (store, product, store_product) and clamping
/// the radius server-side.
///
/// The `top`/`cnt` lateral joins (ranked-top-5 `json_agg` + true distinct
/// count) are duplicated in `search_all_for_map` below with different
/// placeholder numbers — keep both in sync if the ranking logic changes.
pub async fn search(
    pool: &PgPool,
    lat: f64,
    lon: f64,
    distance_km: f64,
    product_id: Option<i64>,
    category_id: Option<i64>,
) -> sqlx::Result<Vec<StoreSearchResult>> {
    let distance_km = distance_km.clamp(0.0, MAX_DISTANCE_KM);
    let rows = sqlx::query_as!(
        SearchRow,
        r#"
        select
            s.id,
            s.name,
            ST_Y(s.position::geometry) as "lat!",
            ST_X(s.position::geometry) as "lon!",
            ST_Distance(s.position, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) as "distance_m!",
            top.products as "products!: Json<Vec<ProductSummary>>",
            cnt.product_total as "product_total!"
        from store s
        left join lateral (
            select coalesce(
                jsonb_agg(
                    jsonb_build_object('name', ranked.name, 'icon', ranked.icon, 'rating_count', ranked.rating_count)
                    order by ranked.rating_count desc, ranked.name asc
                ),
                '[]'
            ) as products
            from (
                select p.name, p.icon, count(r.id) as rating_count
                from store_product sp
                join product p on p.id = sp.product and p.approved and not p.deleted
                left join rating r on r.store_product = sp.id
                where sp.store = s.id and sp.approved and not sp.deleted
                  and ($4::bigint is null or p.id = $4)
                  and ($5::bigint is null or p.category = $5)
                group by p.id, p.name, p.icon
                order by count(r.id) desc, p.name asc
                limit 5
            ) ranked
        ) top on true
        left join lateral (
            select count(distinct p.id) as product_total
            from store_product sp
            join product p on p.id = sp.product and p.approved and not p.deleted
            where sp.store = s.id and sp.approved and not sp.deleted
              and ($4::bigint is null or p.id = $4)
              and ($5::bigint is null or p.category = $5)
        ) cnt on true
        where s.approved and not s.deleted
          and exists (
                select 1 from store_product sp
                join product p on p.id = sp.product and p.approved and not p.deleted
                where sp.store = s.id and sp.approved and not sp.deleted
                  and ($4::bigint is null or p.id = $4)
                  and ($5::bigint is null or p.category = $5)
              )
          and ST_DWithin(s.position, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3::float8 * 1000)
        order by "distance_m!" asc
        "#,
        lon,
        lat,
        distance_km,
        product_id,
        category_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| StoreSearchResult {
            id: r.id,
            name: r.name,
            lat: r.lat,
            lon: r.lon,
            distance_m: r.distance_m,
            products: r.products.0,
            product_total: r.product_total,
        })
        .collect())
}

/// Mirrors `MapStorePin` for `query_as!` — see `SearchRow`'s comment.
struct MapPinRow {
    id: i64,
    name: String,
    lat: f64,
    lon: f64,
    products: Json<Vec<ProductSummary>>,
    product_total: i64,
}

/// The same category/product filter as `search`, minus `ST_DWithin` and
/// the distance-from-origin calculation — every approved, matching store
/// nationwide, for the map's pins. Follow-up to the original store-search
/// capability: the map used to show only whatever `search` returned (so
/// panning/zooming past the "Umkreis" radius revealed nothing), which
/// hid real, nearby-ish stores just outside the currently-selected
/// radius; the results *list* still comes from `search`, only the pins
/// changed.
pub async fn search_all_for_map(
    pool: &PgPool,
    product_id: Option<i64>,
    category_id: Option<i64>,
) -> sqlx::Result<Vec<MapStorePin>> {
    let rows = sqlx::query_as!(
        MapPinRow,
        r#"
        select
            s.id,
            s.name,
            ST_Y(s.position::geometry) as "lat!",
            ST_X(s.position::geometry) as "lon!",
            top.products as "products!: Json<Vec<ProductSummary>>",
            cnt.product_total as "product_total!"
        from store s
        left join lateral (
            select coalesce(
                jsonb_agg(
                    jsonb_build_object('name', ranked.name, 'icon', ranked.icon, 'rating_count', ranked.rating_count)
                    order by ranked.rating_count desc, ranked.name asc
                ),
                '[]'
            ) as products
            from (
                select p.name, p.icon, count(r.id) as rating_count
                from store_product sp
                join product p on p.id = sp.product and p.approved and not p.deleted
                left join rating r on r.store_product = sp.id
                where sp.store = s.id and sp.approved and not sp.deleted
                  and ($1::bigint is null or p.id = $1)
                  and ($2::bigint is null or p.category = $2)
                group by p.id, p.name, p.icon
                order by count(r.id) desc, p.name asc
                limit 5
            ) ranked
        ) top on true
        left join lateral (
            select count(distinct p.id) as product_total
            from store_product sp
            join product p on p.id = sp.product and p.approved and not p.deleted
            where sp.store = s.id and sp.approved and not sp.deleted
              and ($1::bigint is null or p.id = $1)
              and ($2::bigint is null or p.category = $2)
        ) cnt on true
        where s.approved and not s.deleted
          and exists (
                select 1 from store_product sp
                join product p on p.id = sp.product and p.approved and not p.deleted
                where sp.store = s.id and sp.approved and not sp.deleted
                  and ($1::bigint is null or p.id = $1)
                  and ($2::bigint is null or p.category = $2)
              )
        order by s.name asc
        "#,
        product_id,
        category_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MapStorePin {
            id: r.id,
            name: r.name,
            lat: r.lat,
            lon: r.lon,
            products: r.products.0,
            product_total: r.product_total,
        })
        .collect())
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Store>> {
    sqlx::query_as!(
        Store,
        r#"select id, company, name, openinghours,
                  ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                  approved, deleted, created_by, modified_by, created, modified
           from store where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Public-read variant: only returns the store if it's approved and not
/// deleted (content-moderation capability) — used by the detail/search
/// paths; `find` above (unfiltered) is for edit/delete handlers, which
/// operate on any existing row regardless of moderation state.
pub async fn find_public(pool: &PgPool, id: i64) -> sqlx::Result<Option<Store>> {
    sqlx::query_as!(
        Store,
        r#"select id, company, name, openinghours,
                  ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                  approved, deleted, created_by, modified_by, created, modified
           from store where id = $1 and approved and not deleted"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    company: i64,
    name: &str,
    lat: f64,
    lon: f64,
    openinghours: Option<&str>,
    created_by: i64,
) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"insert into store (company, name, position, openinghours, approved, created_by, modified_by)
           values ($1, $2, ST_SetSRID(ST_MakePoint($4, $3), 4326)::geography, $5, false, $6, $6)
           returning id, company, name, openinghours,
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     approved, deleted, created_by, modified_by, created, modified"#,
        company,
        name,
        lat,
        lon,
        openinghours,
        created_by
    )
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: i64,
    company: i64,
    name: &str,
    lat: f64,
    lon: f64,
    openinghours: Option<&str>,
    changed_by: i64,
) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"update store
           set company = $2, name = $3,
               position = ST_SetSRID(ST_MakePoint($5, $4), 4326)::geography,
               openinghours = $6, modified_by = $7, modified = now()
           where id = $1
           returning id, company, name, openinghours,
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     approved, deleted, created_by, modified_by, created, modified"#,
        id,
        company,
        name,
        lat,
        lon,
        openinghours,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"update store set deleted = true, modified_by = $2, modified = now()
           where id = $1
           returning id, company, name, openinghours,
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     approved, deleted, created_by, modified_by, created, modified"#,
        id,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub fn snapshot(store: &Store) -> serde_json::Value {
    json!({
        "id": store.id, "company": store.company, "name": store.name,
        "lat": store.lat, "lon": store.lon, "openinghours": store.openinghours,
        "approved": store.approved, "deleted": store.deleted,
    })
}
