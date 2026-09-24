use serde_json::json;
use sqlx::PgPool;

use crate::models::{AdminProductRow, CatalogProduct};

#[derive(Debug, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub approved: bool,
    pub deleted: bool,
}

/// Everything the filter `<select>` and the forms' product pickers offer.
pub async fn list_all_approved(pool: &PgPool) -> sqlx::Result<Vec<CatalogProduct>> {
    sqlx::query_as!(
        CatalogProduct,
        "select id, name, icon from product where approved and not deleted order by name"
    )
    .fetch_all(pool)
    .await
}

/// The navbar's quick-pick row: ranked by total ratings across every store
/// carrying the product, ties alphabetical. `left join`s keep unrated
/// products in the row (at the bottom).
pub async fn list_top_rated(pool: &PgPool, limit: i64) -> sqlx::Result<Vec<CatalogProduct>> {
    sqlx::query_as!(
        CatalogProduct,
        r#"select p.id, p.name, p.icon
           from product p
           left join store_product sp on sp.product = p.id and sp.approved and not sp.deleted
           left join rating r on r.store_product = sp.id
           where p.approved and not p.deleted
           group by p.id, p.name, p.icon
           order by count(r.id) desc, p.name asc
           limit $1"#,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Product>> {
    sqlx::query_as!(
        Product,
        "select id, name, description, icon, approved, deleted from product where id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

/// A live (non-deleted) product of this name, any case — the unique index
/// is case-sensitive, but "Äpfel" and "äpfel" are the same thing to a
/// shopper.
pub async fn find_live_by_name(pool: &PgPool, name: &str) -> sqlx::Result<Option<Product>> {
    sqlx::query_as!(
        Product,
        "select id, name, description, icon, approved, deleted from product
         where lower(name) = lower($1) and not deleted limit 1",
        name
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &PgPool, name: &str, description: Option<&str>, created_by: i64) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"insert into product (name, description, approved, created_by, modified_by)
           values ($1, $2, false, $3, $3)
           returning id, name, description, icon, approved, deleted"#,
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
    name: &str,
    description: Option<&str>,
    icon: Option<&str>,
    changed_by: i64,
) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"update product set name = $2, description = $3, icon = $4, modified_by = $5, modified = now()
           where id = $1
           returning id, name, description, icon, approved, deleted"#,
        id,
        name,
        description,
        icon,
        changed_by
    )
    .fetch_one(pool)
    .await
}

/// Removes the product from the shared catalog entirely — not one shop's
/// offer of it (that's `store_product::soft_delete`).
pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "update product set deleted = true, modified_by = $2, modified = now() where id = $1",
        id,
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// The admin "existing" tab: the live catalog, with how many stores
/// offer each product (so a delete's reach is known up front).
pub async fn list_live_for_admin(pool: &PgPool) -> sqlx::Result<Vec<AdminProductRow>> {
    sqlx::query_as!(
        AdminProductRow,
        r#"select p.id, p.name, p.description, p.icon,
                  count(sp.id) as "stores!"
           from product p
           left join store_product sp on sp.product = p.id and sp.approved and not sp.deleted
           where p.approved and not p.deleted
           group by p.id
           order by p.name"#
    )
    .fetch_all(pool)
    .await
}

/// Deletes the product and takes it off every store: its live offers are
/// soft-deleted with it, in one transaction. Returns the offers' ids, for
/// the edit log. Restoring the product later doesn't bring them back;
/// each can be restored from the offers' "deleted" tab.
pub async fn soft_delete_with_offers(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<Vec<i64>> {
    let mut tx = pool.begin().await?;
    let offers = sqlx::query_scalar!(
        "update store_product set deleted = true, modified_by = $2, modified = now()
         where product = $1 and not deleted
         returning id",
        id,
        changed_by
    )
    .fetch_all(&mut *tx)
    .await?;
    sqlx::query!(
        "update product set deleted = true, modified_by = $2, modified = now() where id = $1",
        id,
        changed_by
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(offers)
}

pub fn snapshot(product: &Product) -> serde_json::Value {
    json!({
        "id": product.id, "name": product.name, "description": product.description,
        "icon": product.icon, "approved": product.approved, "deleted": product.deleted,
    })
}
