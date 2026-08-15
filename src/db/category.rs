use sqlx::PgPool;

use crate::models::Category;

pub async fn list_all(pool: &PgPool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(Category, "select id, name, icon from category order by name")
        .fetch_all(pool)
        .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Category>> {
    sqlx::query_as!(Category, "select id, name, icon from category where id = $1", id)
        .fetch_optional(pool)
        .await
}

/// Name-substring match for the navbar's global search suggestions
/// (`handlers::search::suggest`). `citext` isn't on this column, so the
/// match is explicitly case-insensitive via `ilike`.
pub async fn search_by_name(pool: &PgPool, term: &str, limit: i64) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(
        Category,
        "select id, name, icon from category where name ilike $1 order by name limit $2",
        super::contains_pattern(term),
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn exists(pool: &PgPool, id: i64) -> sqlx::Result<bool> {
    let exists = sqlx::query_scalar!(
        r#"select exists(select 1 from category where id = $1) as "exists!""#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}
