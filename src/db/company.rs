use serde_json::json;
use sqlx::PgPool;

use crate::models::Company;

pub async fn list_approved(pool: &PgPool) -> sqlx::Result<Vec<Company>> {
    sqlx::query_as!(
        Company,
        r#"select id, name, description, homepage, approved, deleted,
                  created_by, modified_by, created, modified
           from company
           where approved and not deleted
           order by name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Company>> {
    sqlx::query_as!(
        Company,
        r#"select id, name, description, homepage, approved, deleted,
                  created_by, modified_by, created, modified
           from company where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Inserted with `approved = false` regardless of caller — new creations
/// always gate on moderation (content-moderation capability); only
/// edits/deletes of already-approved rows bypass it (catalog-editing).
pub async fn insert(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    homepage: Option<&str>,
    created_by: i64,
) -> sqlx::Result<Company> {
    sqlx::query_as!(
        Company,
        r#"insert into company (name, description, homepage, approved, created_by, modified_by)
           values ($1, $2, $3, false, $4, $4)
           returning id, name, description, homepage, approved, deleted,
                     created_by, modified_by, created, modified"#,
        name,
        description,
        homepage,
        created_by
    )
    .fetch_one(pool)
    .await
}

/// Catalog-editing capability: any logged-in user, live immediately,
/// `approved` untouched. Returns the row before and after so the caller
/// can write the `edit_log` entry.
pub async fn update(
    pool: &PgPool,
    id: i64,
    name: &str,
    description: Option<&str>,
    homepage: Option<&str>,
    changed_by: i64,
) -> sqlx::Result<Company> {
    sqlx::query_as!(
        Company,
        r#"update company
           set name = $2, description = $3, homepage = $4, modified_by = $5, modified = now()
           where id = $1
           returning id, name, description, homepage, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        name,
        description,
        homepage,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<Company> {
    sqlx::query_as!(
        Company,
        r#"update company set deleted = true, modified_by = $2, modified = now()
           where id = $1
           returning id, name, description, homepage, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub fn snapshot(company: &Company) -> serde_json::Value {
    json!({
        "id": company.id, "name": company.name, "description": company.description,
        "homepage": company.homepage, "approved": company.approved, "deleted": company.deleted,
    })
}
