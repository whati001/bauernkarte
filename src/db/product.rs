use serde_json::json;
use sqlx::PgPool;

use crate::models::{Product, RankedProduct};

/// Everything the search filter's product `<select>` offers, and the same
/// set the submission forms let someone pick an existing product from.
pub async fn list_all_approved(pool: &PgPool) -> sqlx::Result<Vec<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product
           where approved and not deleted
           order by name"#,
    )
    .fetch_all(pool)
    .await
}

/// The navbar's quick-pick row: approved products ranked by their total
/// rating count across every store that carries them, ties broken
/// alphabetically. `left join`s throughout so a product nobody has rated
/// yet still appears (at count 0, i.e. sorted alphabetically at the
/// bottom) rather than dropping out of the row entirely.
pub async fn list_top_rated(pool: &PgPool, limit: i64) -> sqlx::Result<Vec<RankedProduct>> {
    sqlx::query_as!(
        RankedProduct,
        r#"select p.id, p.name, p.icon
           from product p
           left join store_product sp
             on sp.product = p.id and sp.approved and not sp.deleted
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

/// Name-substring match over the same approved-and-not-deleted set the
/// filter `<select>` offers — the product half of the navbar's global
/// search suggestions (`handlers::search::suggest`).
pub async fn search_approved_by_name(pool: &PgPool, term: &str, limit: i64) -> sqlx::Result<Vec<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product
           where approved and not deleted and name ilike $1
           order by name
           limit $2"#,
        crate::db::contains_pattern(term),
        limit
    )
    .fetch_all(pool)
    .await
}

/// `find` restricted to what a visitor may actually filter by — the
/// navbar suggestion `id` arrives in a URL, so it can't be assumed to
/// name an approved product just because the dropdown only offers those.
pub async fn find_approved(pool: &PgPool, id: i64) -> sqlx::Result<Option<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product where id = $1 and approved and not deleted"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Product>> {
    sqlx::query_as!(
        Product,
        r#"select id, name, description, icon, approved, deleted,
                  created_by, modified_by, created, modified
           from product where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    created_by: i64,
) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"insert into product (name, description, approved, created_by, modified_by)
           values ($1, $2, false, $3, $3)
           returning id, name, description, icon, approved, deleted,
                     created_by, modified_by, created, modified"#,
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
    changed_by: i64,
) -> sqlx::Result<Product> {
    sqlx::query_as!(
        Product,
        r#"update product
           set name = $2, description = $3, modified_by = $4, modified = now()
           where id = $1
           returning id, name, description, icon, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
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
           returning id, name, description, icon, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub fn snapshot(product: &Product) -> serde_json::Value {
    json!({
        "id": product.id, "name": product.name,
        "description": product.description, "approved": product.approved, "deleted": product.deleted,
    })
}
