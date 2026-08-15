use serde_json::json;
use sqlx::{types::Json, PgPool};

use crate::models::{ProductSummary, SiblingStore, Store, StoreSearchResult};

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
    distance_m: Option<f64>,
    products: Json<Vec<ProductSummary>>,
    product_total: i64,
}

/// Every approved store matching the category/product filter, ranked by
/// distance from `origin` when there is one and alphabetically when
/// there isn't.
///
/// There is deliberately no radius. The "Umkreis" filter this replaced
/// was a second thing to get right before seeing any results, and it hid
/// stores that were only just outside it — worse than useless when the
/// map was already showing those same pins. Nearness is now a *ranking*,
/// not a gate, which is also why this one query serves both the results
/// list and the map's pins (they were previously two near-identical
/// queries that differed only by `ST_DWithin`).
///
/// `origin` is `None` until geolocation actually resolves — sorting by
/// distance from the Austria-centroid fallback would be a meaningless
/// order dressed up as a meaningful one, so the fallback is alphabetical
/// and `distance_m` stays `NULL` (the UI then shows no distance at all
/// rather than a number measured from nowhere).
pub async fn search(
    pool: &PgPool,
    origin: Option<(f64, f64)>,
    product_id: Option<i64>,
    category_id: Option<i64>,
) -> sqlx::Result<Vec<StoreSearchResult>> {
    let (lat, lon) = match origin {
        Some((lat, lon)) => (Some(lat), Some(lon)),
        None => (None, None),
    };
    let rows = sqlx::query_as!(
        SearchRow,
        r#"
        select
            s.id,
            s.name,
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
                  and ($4::bigint is null or p.category = $4)
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
              and ($4::bigint is null or p.category = $4)
        ) cnt on true
        where s.approved and not s.deleted
          and exists (
                select 1 from store_product sp
                join product p on p.id = sp.product and p.approved and not p.deleted
                where sp.store = s.id and sp.approved and not sp.deleted
                  and ($3::bigint is null or p.id = $3)
                  and ($4::bigint is null or p.category = $4)
              )
        order by "distance_m?" asc nulls last, s.name asc
        "#,
        lon,
        lat,
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

/// The company's *other* approved stores — the detail page links to
/// them ("also from this company"). Excludes `store_id` itself, so an
/// empty result means "this is the company's only shop" and the section
/// is skipped entirely.
pub async fn list_siblings(
    pool: &PgPool,
    company_id: i64,
    store_id: i64,
) -> sqlx::Result<Vec<SiblingStore>> {
    sqlx::query_as!(
        SiblingStore,
        r#"select id, name from store
           where company = $1 and id <> $2 and approved and not deleted
           order by name"#,
        company_id,
        store_id
    )
    .fetch_all(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Store>> {
    sqlx::query_as!(
        Store,
        r#"select id, company, name,
                  openinghours as "openinghours: Json<Vec<crate::models::DayHours>>",
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
        r#"select id, company, name,
                  openinghours as "openinghours: Json<Vec<crate::models::DayHours>>",
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
    openinghours: Option<Vec<crate::models::DayHours>>,
    created_by: i64,
) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"insert into store (company, name, position, openinghours, approved, created_by, modified_by)
           values ($1, $2, ST_SetSRID(ST_MakePoint($4, $3), 4326)::geography, $5, false, $6, $6)
           returning id, company, name,
                     openinghours as "openinghours: Json<Vec<crate::models::DayHours>>",
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     approved, deleted, created_by, modified_by, created, modified"#,
        company,
        name,
        lat,
        lon,
        openinghours.map(Json) as _,
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
    openinghours: Option<Vec<crate::models::DayHours>>,
    changed_by: i64,
) -> sqlx::Result<Store> {
    sqlx::query_as!(
        Store,
        r#"update store
           set company = $2, name = $3,
               position = ST_SetSRID(ST_MakePoint($5, $4), 4326)::geography,
               openinghours = $6, modified_by = $7, modified = now()
           where id = $1
           returning id, company, name,
                     openinghours as "openinghours: Json<Vec<crate::models::DayHours>>",
                     ST_Y(position::geometry) as "lat!", ST_X(position::geometry) as "lon!",
                     approved, deleted, created_by, modified_by, created, modified"#,
        id,
        company,
        name,
        lat,
        lon,
        openinghours.map(Json) as _,
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
           returning id, company, name,
                     openinghours as "openinghours: Json<Vec<crate::models::DayHours>>",
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
