//! Photo uploads. The bytes themselves are served by the plain
//! `GET /image/{id}` route (`server::routes::image`).

use dioxus::fullstack::MultipartFormData;
use dioxus::prelude::*;

use crate::api::error::ApiResult;

#[cfg(feature = "server")]
use crate::{
    api::{error::AppError, non_empty},
    server::{auth, db, image_processing::process_upload, pool},
};

/// Raw upload cap, checked before decoding.
#[cfg(feature = "server")]
const MAX_UPLOAD_BYTES: usize = 15 * 1024 * 1024;

/// Multipart form: `file`, optional `description`, and `is_owner` when the
/// photo is a portrait of the person running the shop. Stored pending
/// approval, so the uploader gets a confirmation, not a changed panel.
#[post("/api/store/{store_id}/image", session: tower_sessions::Session)]
pub async fn upload_image(store_id: i64, mut form: MultipartFormData) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    db::store::find(pool(), store_id).await?.ok_or_else(AppError::not_found)?;

    let mut bytes = None;
    let mut description = String::new();
    let mut is_owner = false;
    while let Some(field) = form.next_field().await.map_err(|_| AppError::invalid("error-image-required"))? {
        match field.name().unwrap_or_default() {
            "file" => {
                let data = field.bytes().await.map_err(|_| AppError::invalid("error-image-too-large"))?;
                if data.len() > MAX_UPLOAD_BYTES {
                    return Err(AppError::invalid("error-image-too-large"));
                }
                if !data.is_empty() {
                    bytes = Some(data);
                }
            }
            "description" => description = field.text().await.unwrap_or_default(),
            "is_owner" => is_owner = true,
            _ => {}
        }
    }
    let bytes = bytes.ok_or_else(|| AppError::invalid("error-image-required"))?;

    // Decoding is CPU-bound; keep it off the async workers.
    let processed = tokio::task::spawn_blocking(move || process_upload(&bytes))
        .await
        .map_err(|err| AppError::from(anyhow::anyhow!("image task failed: {err}")))??;
    let kind = if is_owner { "owner" } else { "photo" };
    let id = db::image::insert(
        pool(),
        store_id,
        &processed.bytes,
        processed.mime_type,
        non_empty(&description),
        kind,
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, store_id, image_id = id, kind, size = processed.bytes.len(), "image uploaded");
    Ok(())
}
