use serde_json::json;
use sqlx::{types::Json, PgPool};

use crate::models::{DayHours, ProductSummary, StoreKind, StoreSearchResult};

/// `position` is a PostGIS geography with no sqlx mapping, so every query
/// projects it as `lat`/`lon` via `ST_Y`/`ST_X` instead.
#[derive(Debug, Clone)]
pub struct Store {
    pub id: i64,
    pub name: String,
    /// A `StoreKind::as_str` value (the column's CHECK constraint).
    pub kind: String,
    pub openinghours: Option<Json<Vec<DayHours>>>,
    pub lat: f64,
    pub lon: f64,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub owner_name: Option<String>,
    pub owner_since: Option<i16>,
    pub owner_bio: Option<String>,
    pub approved: bool,
    pub deleted: bool,
}

/// What `insert`/`update` write — already validated.
pub struct StoreWrite<'a> {
    pub name: &'a str,
    pub kind: StoreKind,
    pub lat: f64,
    pub lon: f64,
    pub openinghours: Option<Vec<DayHours>>,
    pub address: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub owner_name: Option<&'a str>,
    pub owner_since: Option<i16>,
    pub owner_bio: Option<&'a str>,
}

struct SearchRow {
    id: i64,
    name: String,
    kind: String,
    lat: f64,
    lon: f64,
    distance_m: Option<f64>,
    products: Json<Vec<ProductSummary>>,
    product_total: i64,
}

/// Every approved store carrying the product filter, nearest first when
/// there's a real geolocation fix and alphabetical when there isn't.
/// There's no radius: nearness is a ranking, not a gate, which is also
/// why one query serves both the results list and the map's pins.
pub async fn search(
    pool: &PgPool,
    origin: Option<(f64, f64)>,
    product_id: Option<i64>,
) -> sqlx::Result<Vec<StoreSearchResult>> {
    let (lat, lon) = origin.map_or((None, None), |(lat, lon)| (Some(lat), Some(lon)));
    let rows = sqlx::query_as!(
        SearchRow,
        r#"
        select
            s.id,
            s.name,
            s.kind,
            ST_Y(s.position::geometry) as "lat!",
            ST_X(s.position::geometry) as "lon!",
            case when $1::float8 is null or $2::float8 is null then null
                 else ST_Distance(s.position, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)
            end as "distance_m?",
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
                  and ($3::bigint is null or p.id = $3)
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
              and ($3::bigint is null or p.id = $3)
        ) cnt on true
        where s.approved and not s.deleted
          and exists (
                select 1 from store_product sp
                join product p on p.id = sp.product and p.approved and not p.deleted
                where sp.store = s.id and sp.approved and not sp.deleted
                  and ($3::bigint is null or p.id = $3)
              )
        order by "distance_m?" asc nulls last, s.name asc
        "#,
        lon,
        lat,
        product_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| StoreSearchResult {
            id: r.id,
            name: r.name,
            kind: StoreKind::from_db(&r.kind),
            lat: r.lat,
            lon: r.lon,
            distance_m: r.distance_m,
            products: r.products.0,
            product_total: r.product_total,
        })
        .collect())
}

/// Unfiltered — for edit/delete, which act on any existing row.
pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Store>> {
    sqlx::query_as!(
        Store,
        r#"select id, name, kind, openinghours as "openinghours: Json<Vec<DayHours>>",
                  ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                  address, phone, owner_name, owner_since, owner_bio,
                  approved, deleted
           from store where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Approved and not deleted only — what a visitor may see.
pub async fn find_public(pool: &PgPool, id: i64) -> sqlx::Result<Option<Store>> {
    Ok(find(pool, id).await?.filter(|s| s.approved && !s.deleted))
}

pub async fn insert(pool: &PgPool, store: &StoreWrite<'_>, created_by: i64) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"insert into store (name, position, openinghours, address, phone,
                              owner_name, owner_since, owner_bio, kind, approved, created_by, modified_by)
           values ($1, ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography, $4, $5, $6, $7, $8, $9, $10, false, $11, $11)
           returning id, name, kind, openinghours as "openinghours: Json<Vec<DayHours>>",
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     address, phone, owner_name, owner_since, owner_bio,
                     approved, deleted"#,
        store.name,
        store.lat,
        store.lon,
        store.openinghours.clone().map(Json) as _,
        store.address,
        store.phone,
        store.owner_name,
        store.owner_since,
        store.owner_bio,
        store.kind.as_str(),
        created_by
    )
    .fetch_one(pool)
    .await
}

/// Catalog editing: live immediately, `approved` untouched.
pub async fn update(pool: &PgPool, id: i64, store: &StoreWrite<'_>, changed_by: i64) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"update store
           set name = $2, position = ST_SetSRID(ST_MakePoint($4, $3), 4326)::geography,
               openinghours = $5, address = $6, phone = $7,
               owner_name = $8, owner_since = $9, owner_bio = $10, kind = $11,
               modified_by = $12, modified = now()
           where id = $1
           returning id, name, kind, openinghours as "openinghours: Json<Vec<DayHours>>",
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     address, phone, owner_name, owner_since, owner_bio,
                     approved, deleted"#,
        id,
        store.name,
        store.lat,
        store.lon,
        store.openinghours.clone().map(Json) as _,
        store.address,
        store.phone,
        store.owner_name,
        store.owner_since,
        store.owner_bio,
        store.kind.as_str(),
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "update store set deleted = true, modified_by = $2, modified = now() where id = $1",
        id,
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// The `edit_log` snapshot. Shaped like `StoreWrite` so a logged edit can
/// be reverted by feeding it back through `update`.
pub fn snapshot(store: &Store) -> serde_json::Value {
    json!({
        "id": store.id, "name": store.name, "kind": store.kind, "lat": store.lat, "lon": store.lon,
        "openinghours": store.openinghours,
        "address": store.address, "phone": store.phone,
        "owner_name": store.owner_name, "owner_since": store.owner_since, "owner_bio": store.owner_bio,
        "approved": store.approved, "deleted": store.deleted,
    })
}
