//! image-upload capability (create + serve) and catalog-editing for
//! `image` (description edit + soft delete; replacing the binary itself
//! is modeled as a new upload + delete of the old one, per design.md).

use askama::Template;
use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use futures_util::stream;
use serde::Deserialize;
use std::convert::Infallible;

use crate::{
    auth::{CurrentUser, OptionalUser},
    db,
    error::{AppError, AppResult},
    handlers::store_detail::load_detail_or_404,
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    image_processing::process_upload,
    sse::patch_elements_at,
    state::AppState,
    templates::render,
};

/// Raw multipart upload cap, checked before decoding (image-upload spec's
/// "reject oversized upload before processing it").
const MAX_UPLOAD_BYTES: usize = 15 * 1024 * 1024;

async fn detail_html_for_store_product(
    state: &AppState,
    store_product_id: i64,
    viewer_id: i64,
) -> AppResult<String> {
    let sp = db::store_product::find(&state.pool, store_product_id).await?.ok_or(AppError::NotFound)?;
    let detail = load_detail_or_404(state, sp.store, Some(viewer_id)).await?;
    Ok(crate::handlers::store_detail::render_detail_panel(&detail, true))
}

#[derive(Template)]
#[template(path = "partials/image_form.html")]
struct ImageFormTemplate {
    store_product_id: i64,
}

/// `GET /store-product/{id}/image/new` (image-upload capability).
pub async fn new_form(
    Path(store_product_id): Path<i64>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let html = render(ImageFormTemplate { store_product_id });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `POST /store-product/{id}/image` — multipart upload -> `image` row,
/// `approved=false` (image-upload capability).
pub async fn upload(
    State(state): State<AppState>,
    Path(store_product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    mut multipart: Multipart,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    if db::store_product::find(&state.pool, store_product_id).await?.is_none() {
        return Err(AppError::NotFound);
    }

    let mut file_bytes: Option<Bytes> = None;
    let mut description: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("Ungültiger Upload.".into()))?
    {
        match field.name().unwrap_or_default() {
            "file" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::Validation("Datei konnte nicht gelesen werden.".into()))?;
                if data.len() > MAX_UPLOAD_BYTES {
                    return Err(AppError::Validation("Die Datei ist zu groß (max. 15 MB).".into()));
                }
                file_bytes = Some(data);
            }
            "description" => {
                description = field.text().await.ok().filter(|s| !s.trim().is_empty());
            }
            _ => {}
        }
    }

    let Some(file_bytes) = file_bytes else {
        return Err(AppError::Validation("Bitte eine Bilddatei auswählen.".into()));
    };

    let processed = process_upload(&file_bytes)?;
    db::image::insert(
        &state.pool,
        store_product_id,
        &processed.bytes,
        processed.mime_type,
        description.as_deref(),
        user.id,
    )
    .await?;

    // A new image is `approved=false` (community-submissions — moderation
    // matters more for arbitrary uploaded files than for a text edit), so
    // re-rendering the detail panel here (as this used to) would show
    // *no visible change at all*: the just-uploaded image is filtered out
    // of `db::image::list_for_store_product` until approved, same as a
    // brand-new store/product doesn't appear in search yet. Every other
    // pending-review creation flow (store.rs::create, product.rs::create)
    // shows an explicit confirmation instead of silently returning to a
    // seemingly-unchanged view — this matches that, rather than leaving
    // the uploader wondering if anything happened.
    let html = crate::templates::render_confirmation(&i18n::translate(
        i18n::current_locale(),
        "confirmation-image-pending",
    ));
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `GET /image/{id}` — raw bytes, `Content-Type` from `mime_type`, only
/// if approved-and-not-deleted or the requester is the uploader
/// (image-upload capability).
pub async fn show(
    State(state): State<AppState>,
    Path(image_id): Path<i64>,
    OptionalUser(user): OptionalUser,
) -> AppResult<Response> {
    let image = db::image::find(&state.pool, image_id).await?.ok_or(AppError::NotFound)?;

    let is_owner = user.as_ref().is_some_and(|u| Some(u.id) == image.created_by);
    let publicly_visible = image.approved && !image.deleted;
    if !publicly_visible && !is_owner {
        return Err(AppError::NotFound);
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, image.mime_type.clone())],
        image.image,
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct EditImageBody {
    #[serde(default)]
    description: Option<String>,
}

/// `PATCH /image/{id}` — description only (catalog-editing capability).
pub async fn update(
    State(state): State<AppState>,
    Path(image_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<EditImageBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::image::find(&state.pool, image_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("image is deleted".into()));
    }
    let old_snapshot = db::image::snapshot(&before);
    let description = body.description.filter(|s| !s.trim().is_empty());
    let after = db::image::update_description(&state.pool, image_id, description.as_deref(), user.id).await?;
    let new_snapshot = db::image::snapshot(&after);
    db::edit_log::write(
        &state.pool,
        "image",
        image_id,
        db::edit_log::EditAction::Update,
        &old_snapshot,
        Some(&new_snapshot),
        user.id,
    )
    .await?;

    let html = detail_html_for_store_product(&state, before.store_product, user.id).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `DELETE /image/{id}` (catalog-editing capability, soft delete).
pub async fn delete(
    State(state): State<AppState>,
    Path(image_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::image::find(&state.pool, image_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("image already deleted".into()));
    }
    let old_snapshot = db::image::snapshot(&before);
    db::image::soft_delete(&state.pool, image_id, user.id).await?;
    db::edit_log::write(
        &state.pool,
        "image",
        image_id,
        db::edit_log::EditAction::Delete,
        &old_snapshot,
        None,
        user.id,
    )
    .await?;

    let html = detail_html_for_store_product(&state, before.store_product, user.id).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}
