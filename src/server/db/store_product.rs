use serde_json::json;
use sqlx::{types::Json, PgPool};

/// "This shop sells this product", and in which months.
#[derive(Debug, Clone)]
pub struct StoreProduct {
    pub id: i64,
    pub store: i64,
    pub product: i64,
    pub seasonal_months: Option<Json<Vec<i16>>>,
    pub approved: bool,
    pub deleted: bool,
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<StoreProduct>> {
    sqlx::query_as!(
        StoreProduct,
        r#"select id, store, product, seasonal_months as "seasonal_months: Json<Vec<i16>>",
                  approved, deleted
           from store_product where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    store: i64,
    product: i64,
    seasonal_months: Option<Vec<i16>>,
    created_by: i64,
) -> sqlx::Result<StoreProduct> {
    sqlx::query_as!(
        StoreProduct,
        r#"insert into store_product (store, product, seasonal_months, approved, created_by, modified_by)
           values ($1, $2, $3, false, $4, $4)
           returning id, store, product, seasonal_months as "seasonal_months: Json<Vec<i16>>",
                     approved, deleted"#,
        store,
        product,
        seasonal_months.map(Json) as _,
        created_by
    )
    .fetch_one(pool)
    .await
}

/// Catalog editing: `approved`/`deleted` untouched.
pub async fn update_seasonality(
    pool: &PgPool,
    id: i64,
    seasonal_months: Option<Vec<i16>>,
    changed_by: i64,
) -> sqlx::Result<StoreProduct> {
    sqlx::query_as!(
        StoreProduct,
        r#"update store_product set seasonal_months = $2, modified_by = $3, modified = now()
           where id = $1
           returning id, store, product, seasonal_months as "seasonal_months: Json<Vec<i16>>",
                     approved, deleted"#,
        id,
        seasonal_months.map(Json) as _,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "update store_product set deleted = true, modified_by = $2, modified = now() where id = $1",
        id,
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub fn snapshot(sp: &StoreProduct) -> serde_json::Value {
    json!({
        "id": sp.id, "store": sp.store, "product": sp.product,
        "seasonal_months": sp.seasonal_months,
        "approved": sp.approved, "deleted": sp.deleted,
    })
}
