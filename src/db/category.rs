use sqlx::PgPool;

use crate::models::Category;

pub async fn list_all(pool: &PgPool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(Category, "select id, name, icon from category order by name")
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
