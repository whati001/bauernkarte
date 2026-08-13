use serde_json::json;
use sqlx::PgPool;

use crate::models::StoreProduct;

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<StoreProduct>> {
    sqlx::query_as!(
        StoreProduct,
        r#"select id, store, product, approved, deleted,
                  created_by, modified_by, created, modified
           from store_product where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &PgPool, store: i64, product: i64, created_by: i64) -> sqlx::Result<StoreProduct> {
    sqlx::query_as!(
        StoreProduct,
        r#"insert into store_product (store, product, approved, created_by, modified_by)
           values ($1, $2, false, $3, $3)
           returning id, store, product, approved, deleted,
                     created_by, modified_by, created, modified"#,
        store,
        product,
        created_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<StoreProduct> {
    sqlx::query_as!(
        StoreProduct,
        r#"update store_product set deleted = true, modified_by = $2, modified = now()
           where id = $1
           returning id, store, product, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub fn snapshot(sp: &StoreProduct) -> serde_json::Value {
    json!({
        "id": sp.id, "store": sp.store, "product": sp.product,
        "approved": sp.approved, "deleted": sp.deleted,
    })
}
