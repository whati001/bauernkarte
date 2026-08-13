use serde_json::json;
use sqlx::PgPool;

use crate::models::{Image, ImageSummary};

pub async fn list_for_store_product(pool: &PgPool, store_product_id: i64) -> sqlx::Result<Vec<ImageSummary>> {
    sqlx::query_as!(
        ImageSummary,
        r#"select id, description from image
           where store_product = $1 and approved and not deleted
           order by created"#,
        store_product_id
    )
    .fetch_all(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Image>> {
    sqlx::query_as!(
        Image,
        r#"select id, store_product, image, mime_type, description, approved, deleted,
                  created_by, modified_by, created, modified
           from image where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &PgPool,
    store_product_id: i64,
    image_bytes: &[u8],
    mime_type: &str,
    description: Option<&str>,
    created_by: i64,
) -> sqlx::Result<Image> {
    sqlx::query_as!(
        Image,
        r#"insert into image (store_product, image, mime_type, description, approved, created_by, modified_by)
           values ($1, $2, $3, $4, false, $5, $5)
           returning id, store_product, image, mime_type, description, approved, deleted,
                     created_by, modified_by, created, modified"#,
        store_product_id,
        image_bytes,
        mime_type,
        description,
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
) -> sqlx::Result<Image> {
    sqlx::query_as!(
        Image,
        r#"update image set description = $2, modified_by = $3, modified = now()
           where id = $1
           returning id, store_product, image, mime_type, description, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        description,
        changed_by
    )
    .fetch_one(pool)
    .await
}

pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<Image> {
    sqlx::query_as!(
        Image,
        r#"update image set deleted = true, modified_by = $2, modified = now()
           where id = $1
           returning id, store_product, image, mime_type, description, approved, deleted,
                     created_by, modified_by, created, modified"#,
        id,
        changed_by
    )
    .fetch_one(pool)
    .await
}

/// Snapshot for `edit_log` — deliberately excludes the image bytes
/// themselves (`image` column): a multi-hundred-KB blob doesn't belong in
/// an audit row meant to be read/diffed by a human, and image replacement
/// is already modeled as upload-new + delete-old (each independently
/// logged), so the bytes are never actually "edited" in place.
pub fn snapshot(image: &Image) -> serde_json::Value {
    json!({
        "id": image.id, "store_product": image.store_product, "mime_type": image.mime_type,
        "description": image.description, "approved": image.approved, "deleted": image.deleted,
    })
}
