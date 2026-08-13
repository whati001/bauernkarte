use sqlx::PgPool;

use crate::models::RatingCount;

pub async fn default_rating_type_id(pool: &PgPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"select id from rating_type where name = 'UP'"#)
        .fetch_one(pool)
        .await
}

/// Ratings capability: a toggle, not a stackable counter — `ON CONFLICT
/// DO NOTHING` on the `(store_product, created_by, rating_type)` unique
/// index makes re-rating a no-op rather than an error or a second row.
pub async fn upsert(pool: &PgPool, store_product_id: i64, rating_type_id: i64, user_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        r#"insert into rating (store_product, rating_type, created_by)
           values ($1, $2, $3)
           on conflict (store_product, created_by, rating_type) do nothing"#,
        store_product_id,
        rating_type_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Returns whether a row was actually deleted, so the caller can 403 when
/// the requester doesn't own the rating (as opposed to it simply not
/// existing, which is a 404) — the ratings spec requires owner-only
/// removal.
pub async fn delete_owned(pool: &PgPool, rating_id: i64, user_id: i64) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "delete from rating where id = $1 and created_by = $2",
        rating_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn counts_for_store_product(pool: &PgPool, store_product_id: i64) -> sqlx::Result<Vec<RatingCount>> {
    sqlx::query_as!(
        RatingCount,
        r#"select rt.id as rating_type_id, rt.name as rating_type_name, count(r.id) as "count!"
           from rating_type rt
           left join rating r on r.rating_type = rt.id and r.store_product = $1
           group by rt.id, rt.name
           having count(r.id) > 0
           order by rt.id"#,
        store_product_id
    )
    .fetch_all(pool)
    .await
}

pub async fn viewer_has_rated_up(pool: &PgPool, store_product_id: i64, user_id: i64) -> sqlx::Result<bool> {
    let exists = sqlx::query_scalar!(
        r#"select exists(
             select 1 from rating r
             join rating_type rt on rt.id = r.rating_type and rt.name = 'UP'
             where r.store_product = $1 and r.created_by = $2
           ) as "exists!""#,
        store_product_id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}
