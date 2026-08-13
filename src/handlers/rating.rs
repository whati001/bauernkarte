//! ratings capability. Path-based (`/store-product/{id}/rating`) rather
//! than the `POST /rating` + `DELETE /rating/{id}` shape sketched in
//! design.md's route table: the client only ever knows a store_product's
//! id (from the page it's already rendering), never the underlying
//! rating row's id, so keying off store_product avoids making the client
//! track a second id purely to unrate something.

use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
};
use futures_util::stream;
use std::convert::Infallible;

use crate::{
    auth::CurrentUser,
    db,
    error::{AppError, AppResult},
    handlers::store_detail::load_detail_or_404,
    sse::patch_elements_at,
    state::AppState,
};

async fn detail_html_for_store_product(
    state: &AppState,
    store_product_id: i64,
    viewer_id: i64,
) -> AppResult<String> {
    let sp = db::store_product::find(&state.pool, store_product_id).await?.ok_or(AppError::NotFound)?;
    let detail = load_detail_or_404(state, sp.store, Some(viewer_id)).await?;
    Ok(crate::handlers::store_detail::render_detail_panel(&detail, true))
}

/// `POST /store-product/{id}/rating` — upsert the current user's `UP`
/// rating (idempotent: re-rating is a no-op, not a second row).
pub async fn rate_up(
    State(state): State<AppState>,
    Path(store_product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let rating_type_id = db::rating::default_rating_type_id(&state.pool).await?;
    db::rating::upsert(&state.pool, store_product_id, rating_type_id, user.id).await?;

    let html = detail_html_for_store_product(&state, store_product_id, user.id).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `DELETE /store-product/{id}/rating` — remove the current user's own
/// `UP` rating (owner-only; there's nothing to 403 here since the query
/// is inherently scoped to `created_by = current user`).
pub async fn unrate(
    State(state): State<AppState>,
    Path(store_product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let rating_type_id = db::rating::default_rating_type_id(&state.pool).await?;
    // Look up the rating id owned by this user/type/store_product, then
    // delete via the owner-checked path (keeps a single `delete_owned`
    // entry point instead of a second raw query here).
    let rating_id = sqlx::query_scalar!(
        "select id from rating where store_product = $1 and rating_type = $2 and created_by = $3",
        store_product_id,
        rating_type_id,
        user.id
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(rating_id) = rating_id {
        db::rating::delete_owned(&state.pool, rating_id, user.id).await?;
    }

    let html = detail_html_for_store_product(&state, store_product_id, user.id).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}
