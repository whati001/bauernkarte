//! Per-offer "UP" hearts. A toggle, not a counter: one per user per offer.

use sqlx::PgPool;

pub async fn rate_up(pool: &PgPool, store_product_id: i64, user_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        r#"insert into rating (store_product, rating_type, created_by)
           select $1, rt.id, $2 from rating_type rt where rt.name = 'UP'
           on conflict (store_product, created_by, rating_type) do nothing"#,
        store_product_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Owner-only by construction: scoped to `created_by = user_id`.
pub async fn unrate(pool: &PgPool, store_product_id: i64, user_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        r#"delete from rating r using rating_type rt
           where r.rating_type = rt.id and rt.name = 'UP'
             and r.store_product = $1 and r.created_by = $2"#,
        store_product_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
