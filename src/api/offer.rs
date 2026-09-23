//! Offers — "this shop sells this product", and in which months — and
//! their per-offer hearts.

use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::{OfferEdit, OfferInput};

#[cfg(feature = "server")]
use crate::{
    api::{error::AppError, non_empty},
    models::ProductChoice,
    seasonality,
    server::{
        auth,
        db::{self, edit_log::EditAction, product::Product},
        pool,
    },
};

/// A submitted offer's months, validated.
#[cfg(feature = "server")]
pub(crate) fn parse_offer(offer: &OfferInput) -> ApiResult<Option<Vec<i16>>> {
    seasonality::validate(offer.seasonal_months.clone()).map_err(AppError::invalid)
}

/// The catalog product an offer is for: an existing one, or a new one
/// submitted for approval. A "new" name that already exists (any case) is
/// quietly the existing product — two catalog rows for "Äpfel" helps
/// nobody, and the unique index would reject it anyway.
#[cfg(feature = "server")]
pub(crate) async fn resolve_product(choice: &ProductChoice, user_id: i64) -> ApiResult<Product> {
    match choice {
        ProductChoice::Existing(id) => {
            let product = db::product::find(pool(), *id).await?.ok_or_else(|| AppError::invalid("error-product-required"))?;
            if product.deleted {
                return Err(AppError::invalid("error-product-required"));
            }
            Ok(product)
        }
        ProductChoice::New { name, description } => {
            let name = non_empty(name).ok_or_else(|| AppError::invalid("error-product-name-required"))?;
            if let Some(existing) = db::product::find_live_by_name(pool(), name).await? {
                return Ok(existing);
            }
            Ok(db::product::insert(pool(), name, non_empty(description), user_id).await?)
        }
    }
}

/// Adds a product to an existing store (pending approval). Returns the
/// product's name for the confirmation.
#[post("/api/store/{store_id}/offer", session: tower_sessions::Session)]
pub async fn add_offer(store_id: i64, offer: OfferInput) -> ApiResult<String> {
    let user = auth::require_user(&session).await?;
    db::store::find_public(pool(), store_id).await?.ok_or_else(AppError::not_found)?;
    let months = parse_offer(&offer)?;
    let product = resolve_product(&offer.product, user.id).await?;
    db::store_product::insert(pool(), store_id, product.id, months, user.id).await?;
    tracing::info!(user_id = %user.id, store_id, product_id = product.id, "offer submitted for review");
    Ok(product.name)
}

#[get("/api/offer/{id}", session: tower_sessions::Session)]
pub async fn offer_for_edit(id: i64) -> ApiResult<OfferEdit> {
    auth::require_user(&session).await?;
    let sp = db::store_product::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if sp.deleted {
        return Err(AppError::deleted());
    }
    let product = db::product::find(pool(), sp.product).await?.ok_or_else(AppError::not_found)?;
    Ok(OfferEdit {
        store_product_id: sp.id,
        store_id: sp.store,
        product_name: product.name,
        seasonal_months: sp.seasonal_months.map(|j| j.0),
    })
}

/// The months one offer is available. Returns the store id, for the way
/// back to its panel.
#[patch("/api/offer/{id}", session: tower_sessions::Session)]
pub async fn update_offer(id: i64, seasonal_months: Option<Vec<i16>>) -> ApiResult<i64> {
    let user = auth::require_user(&session).await?;
    let before = db::store_product::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if before.deleted {
        return Err(AppError::deleted());
    }
    let months = seasonality::validate(seasonal_months).map_err(AppError::invalid)?;
    let after = db::store_product::update_seasonality(pool(), id, months, user.id).await?;
    db::edit_log::write(
        pool(),
        "store_product",
        id,
        EditAction::Update,
        &db::store_product::snapshot(&before),
        Some(&db::store_product::snapshot(&after)),
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, store_product_id = id, "seasonality updated");
    Ok(before.store)
}

#[delete("/api/offer/{id}", session: tower_sessions::Session)]
pub async fn delete_offer(id: i64) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    let before = db::store_product::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if before.deleted {
        return Err(AppError::deleted());
    }
    db::store_product::soft_delete(pool(), id, user.id).await?;
    db::edit_log::write(
        pool(),
        "store_product",
        id,
        EditAction::Delete,
        &db::store_product::snapshot(&before),
        None,
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, store_product_id = id, "offer removed");
    Ok(())
}

#[post("/api/offer/{id}/heart", session: tower_sessions::Session)]
pub async fn heart_offer(id: i64) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    db::rating::rate_up(pool(), id, user.id).await?;
    Ok(())
}

#[delete("/api/offer/{id}/heart", session: tower_sessions::Session)]
pub async fn unheart_offer(id: i64) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    db::rating::unrate(pool(), id, user.id).await?;
    Ok(())
}
