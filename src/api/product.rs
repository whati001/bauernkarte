//! Catalog products themselves (shared by every store that offers them).

use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::ProductEdit;

#[cfg(feature = "server")]
use crate::{
    api::{error::AppError, non_empty},
    server::{
        auth,
        db::{self, edit_log::EditAction},
        pool,
    },
};

#[get("/api/product/{id}", session: tower_sessions::Session)]
pub async fn product_for_edit(id: i64) -> ApiResult<ProductEdit> {
    auth::require_user(&session).await?;
    let p = db::product::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if p.deleted {
        return Err(AppError::deleted());
    }
    Ok(ProductEdit { id: p.id, name: p.name, description: p.description })
}

/// Returns the saved name, for the confirmation.
#[patch("/api/product/{id}", session: tower_sessions::Session)]
pub async fn update_product(id: i64, name: String, description: String) -> ApiResult<String> {
    let user = auth::require_user(&session).await?;
    let before = db::product::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if before.deleted {
        return Err(AppError::deleted());
    }
    let name = non_empty(&name).ok_or_else(|| AppError::invalid("error-name-required"))?;
    if db::product::find_live_by_name(pool(), name).await?.is_some_and(|other| other.id != id) {
        return Err(AppError::invalid("error-product-name-taken"));
    }
    let after = db::product::update(pool(), id, name, non_empty(&description), before.icon.as_deref(), user.id).await?;
    db::edit_log::write(
        pool(),
        "product",
        id,
        EditAction::Update,
        &db::product::snapshot(&before),
        Some(&db::product::snapshot(&after)),
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, product_id = id, "product updated");
    Ok(after.name)
}

/// Removes the product from the whole catalog, not one shop's offer.
#[delete("/api/product/{id}", session: tower_sessions::Session)]
pub async fn delete_product(id: i64) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    let before = db::product::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if before.deleted {
        return Err(AppError::deleted());
    }
    db::product::soft_delete(pool(), id, user.id).await?;
    db::edit_log::write(pool(), "product", id, EditAction::Delete, &db::product::snapshot(&before), None, user.id)
        .await?;
    tracing::info!(user_id = %user.id, product_id = id, "product deleted");
    Ok(())
}
