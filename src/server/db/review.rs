//! The shop's 1–5 star rating: one review per user per shop, replaced
//! when the same user rates again.

use sqlx::PgPool;

use crate::models::ReviewSummary;

pub async fn summary(pool: &PgPool, store_id: i64, viewer_id: Option<i64>) -> sqlx::Result<ReviewSummary> {
    let row = sqlx::query!(
        r#"select avg(stars)::float8 as average, count(*) as "count!",
                  max(stars) filter (where created_by = $2) as mine
           from store_review where store = $1"#,
        store_id,
        viewer_id
    )
    .fetch_one(pool)
    .await?;
    Ok(ReviewSummary { average: row.average, count: row.count, mine: row.mine })
}

pub async fn upsert(pool: &PgPool, store_id: i64, user_id: i64, stars: i16) -> sqlx::Result<()> {
    sqlx::query!(
        r#"insert into store_review (store, stars, created_by) values ($1, $2, $3)
           on conflict (store, created_by) do update set stars = excluded.stars, modified = now()"#,
        store_id,
        stars,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_own(pool: &PgPool, store_id: i64, user_id: i64) -> sqlx::Result<()> {
    sqlx::query!("delete from store_review where store = $1 and created_by = $2", store_id, user_id)
        .execute(pool)
        .await?;
    Ok(())
}
