use serde_json::json;
use sqlx::PgPool;

use crate::models::{AdminImageRow, ImageSummary};

#[derive(Debug, Clone)]
pub struct Image {
    pub id: i64,
    pub store: i64,
    pub image: Vec<u8>,
    pub mime_type: String,
    pub description: Option<String>,
    pub kind: String,
    /// Picked by the uploader as the store image.
    pub cover: bool,
    pub approved: bool,
    pub deleted: bool,
    pub created_by: Option<i64>,
}

/// Photos of the place — portraits are the owner block's, not the
/// carousel's. The store image comes first (the panel's header shows
/// it): the newest one marked as such, else the oldest photo. Same order
/// as the search list's thumbnail (`store::search`).
pub async fn list_photos(pool: &PgPool, store_id: i64) -> sqlx::Result<Vec<ImageSummary>> {
    sqlx::query_as!(
        ImageSummary,
        r#"select id, description from image
           where store = $1 and kind = 'photo' and approved and not deleted
           order by (case when cover then id end) desc nulls last, id"#,
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
        r#"select id, store, image, mime_type, description, kind, cover, approved, deleted, created_by
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
    cover: bool,
    created_by: i64,
) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"insert into image (store, image, mime_type, description, kind, cover, approved, created_by, modified_by)
           values ($1, $2, $3, $4, $5, $6, false, $7, $7)
           returning id"#,
        store_id,
        bytes,
        mime_type,
        description,
        kind,
        cover,
        created_by
    )
    .fetch_one(pool)
    .await
}

/// The admin "existing" tab: every live image, newest first, with its
/// store. Portraits included (the owner block shows them).
pub async fn list_live_for_admin(pool: &PgPool) -> sqlx::Result<Vec<AdminImageRow>> {
    sqlx::query_as!(
        AdminImageRow,
        r#"select i.id, i.store as store_id, s.name as store_name, i.description,
                  i.kind = 'owner' as "is_owner!", i.cover
           from image i join store s on s.id = i.store
           where i.approved and not i.deleted
           order by i.id desc"#
    )
    .fetch_all(pool)
    .await
}

/// The admin edit: description and the store-image flag. A portrait
/// never becomes the store image.
pub async fn update(pool: &PgPool, id: i64, description: Option<&str>, cover: bool, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "update image set description = $2, cover = $3 and kind = 'photo', modified_by = $4, modified = now()
         where id = $1",
        id,
        description,
        cover,
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Gone from the store panel and the search list, which only show live
/// images; restorable from the admin "deleted" tab.
pub async fn soft_delete(pool: &PgPool, id: i64, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "update image set deleted = true, modified_by = $2, modified = now() where id = $1",
        id,
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
        "description": image.description, "kind": image.kind, "cover": image.cover,
        "approved": image.approved, "deleted": image.deleted,
    })
}
