use serde_json::json;
use sqlx::PgPool;

use crate::models::Product;

/// Every approved product, regardless of category — the search filter's
/// default state (no category selected yet). `list_approved_by_category`
/// below narrows this once a category is picked (store-search capability:
/// "product options cascade from the selected category").
pub async fn list_all_approved(pool: &PgPool) -> sqlx::Result<Vec<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, category, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product
           where approved and not deleted
           order by name"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_approved_by_category(pool: &PgPool, category_id: i64) -> sqlx::Result<Vec<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, category, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product
           where approved and not deleted and category = $1
           order by name"#,
        category_id
    )
    .fetch_all(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, category, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    category: i64,
    name: &str,
    description: Option<&str>,
    created_by: i64,
) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"insert into product (category, name, description, approved, created_by, modified_by)
           values ($1, $2, $3, false, $4, $4)
           returning id, category, name, description, icon, approved, deleted,
                     created_by, modified_by, created, modified"#,
        category,
        name,
        description,
        created_by
    )
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: i64,
    category: i64,
    name: &str,
    description: Option<&str>,
    changed_by: i64,
) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"update product
           set category = $2, name = $3, description = $4, modified_by = $5, modified = now()
           where id = $1
           returning id, category, name, description, icon, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        category,
        name,
        description,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"update product set deleted = true, modified_by = $2, modified = now()
           where id = $1
           returning id, category, name, description, icon, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub fn snapshot(product: &Product) -> serde_json::Value {
    json!({
        "id": product.id, "category": product.category, "name": product.name,
        "description": product.description, "approved": product.approved, "deleted": product.deleted,
    })
}
