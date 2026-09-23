use serde_json::json;
use sqlx::PgPool;

use crate::models::ImageSummary;

#[derive(Debug, Clone)]
pub struct Image {
    pub id: i64,
    pub store: i64,
    pub image: Vec<u8>,
    pub mime_type: String,
    pub description: Option<String>,
    pub kind: String,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
}

/// Photos of the place — portraits are the owner block's, not the
/// carousel's.
pub async fn list_photos(pool: &PgPool, store_id: i64) -> sqlx::Result<Vec<ImageSummary>> {
    sqlx::query_as!(
        ImageSummary,
        r#"select id, description from image
           where store = $1 and kind = 'photo' and approved and not deleted
           order by created"#,
        store_id
    )
    .fetch_all(pool)
    .await
}

/// The newest approved owner portrait.
pub async fn owner_portrait(pool: &PgPool, store_id: i64) -> sqlx::Result<Option<i64>> {
    sqlx::query_scalar!(
        r#"select id from image
           where store = $1 and kind = 'owner' and approved and not deleted
           order by created desc limit 1"#,
        store_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Image>> {
    sqlx::query_as!(
        Image,
        r#"select id, store, image, mime_type, description, kind, approved, deleted, created_by
           from image where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    store_id: i64,
    bytes: &[u8],
    mime_type: &str,
    description: Option<&str>,
    kind: &str,
    created_by: i64,
) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"insert into image (store, image, mime_type, description, kind, approved, created_by, modified_by)
           values ($1, $2, $3, $4, $5, false, $6, $6)
           returning id"#,
        store_id,
        bytes,
        mime_type,
        description,
        kind,
        created_by
    )
    .fetch_one(pool)
    .await
}

pub async fn update_description(
    pool: &PgPool,
    id: i64,
    description: Option<&str>,
    changed_by: i64,
) -> sqlx::Result<()> {
    sqlx::query!(
        "update image set description = $2, modified_by = $3, modified = now() where id = $1",
        id,
        description,
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// For `edit_log` — without the bytes, which don't belong in an audit row
/// and are never edited in place anyway.
pub fn snapshot(image: &Image) -> serde_json::Value {
    json!({
        "id": image.id, "store": image.store, "mime_type": image.mime_type,
        "description": image.description, "kind": image.kind,
        "approved": image.approved, "deleted": image.deleted,
    })
}
