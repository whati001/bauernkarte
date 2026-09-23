//! A store: its info panel, create/edit/delete, and its star rating.

use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::{OfferInput, ReviewSummary, StoreDetail, StoreFields};

#[cfg(feature = "server")]
use crate::{
    api::{error::AppError, non_empty, offer::{parse_offer, resolve_product}},
    models::StoreKind,
    opening_hours,
    server::{
        auth,
        db::{self, edit_log::EditAction, store::StoreWrite},
        pool,
    },
};

#[get("/api/store/{id}", session: tower_sessions::Session)]
pub async fn store_detail(id: i64) -> ApiResult<StoreDetail> {
    let viewer = auth::current_user(&session).await?;
    db::detail::get_store_detail(pool(), id, viewer.map(|u| u.id))
        .await?
        .ok_or_else(AppError::not_found)
}

/// The edit form's starting values.
#[get("/api/store/{id}/edit", session: tower_sessions::Session)]
pub async fn store_for_edit(id: i64) -> ApiResult<StoreFields> {
    auth::require_user(&session).await?;
    let s = db::store::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if s.deleted {
        return Err(AppError::deleted());
    }
    Ok(StoreFields {
        name: s.name,
        kind: StoreKind::from_db(&s.kind),
        lat: Some(s.lat),
        lon: Some(s.lon),
        openinghours: s.openinghours.map(|j| j.0).unwrap_or_default(),
        address: s.address.unwrap_or_default(),
        phone: s.phone.unwrap_or_default(),
        owner_name: s.owner_name.unwrap_or_default(),
        owner_since: s.owner_since.map(|y| y.to_string()).unwrap_or_default(),
        owner_bio: s.owner_bio.unwrap_or_default(),
    })
}

/// Validates a submitted form into what the database layer writes.
#[cfg(feature = "server")]
fn parse_store(fields: &StoreFields) -> ApiResult<StoreWrite<'_>> {
    let name = non_empty(&fields.name).ok_or_else(|| AppError::invalid("error-name-required"))?;
    let (Some(lat), Some(lon)) = (fields.lat, fields.lon) else {
        return Err(AppError::invalid("error-location-required"));
    };
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err(AppError::invalid("error-location-required"));
    }
    let hours = opening_hours::validate(&fields.openinghours).map_err(AppError::invalid)?;
    let owner_since = match non_empty(&fields.owner_since) {
        None => None,
        Some(text) => {
            let this_year = time::OffsetDateTime::now_utc().year() as i16;
            let year = text.parse::<i16>().ok().filter(|y| (1800..=this_year).contains(y));
            Some(year.ok_or_else(|| AppError::invalid("error-owner-since-invalid"))?)
        }
    };
    Ok(StoreWrite {
        name,
        kind: fields.kind,
        lat,
        lon,
        openinghours: (!hours.is_empty()).then_some(hours),
        address: non_empty(&fields.address),
        phone: non_empty(&fields.phone),
        owner_name: non_empty(&fields.owner_name),
        owner_since,
        owner_bio: non_empty(&fields.owner_bio),
    })
}

/// A new store, awaiting approval, with at least one offer: a store with
/// none is invisible to search, which would look like a bug. Everything is
/// validated before anything is written, so a bad third offer can't leave
/// a store and two offers behind. Returns the name for the confirmation.
#[post("/api/store", session: tower_sessions::Session)]
pub async fn create_store(fields: StoreFields, offers: Vec<OfferInput>) -> ApiResult<String> {
    let user = auth::require_user(&session).await?;
    let store = parse_store(&fields)?;
    if offers.is_empty() {
        return Err(AppError::invalid("error-products-min-one"));
    }
    let parsed = offers.iter().map(parse_offer).collect::<ApiResult<Vec<_>>>()?;
    let mut products = Vec::with_capacity(offers.len());
    for offer in &offers {
        products.push(resolve_product(&offer.product, user.id).await?);
    }

    let created = db::store::insert(pool(), &store, user.id).await?;
    for (product, months) in products.iter().zip(parsed) {
        db::store_product::insert(pool(), created.id, product.id, months, user.id).await?;
    }
    tracing::info!(user_id = %user.id, store_id = %created.id, offers = products.len(), "store submitted for review");
    Ok(created.name)
}

/// Catalog editing: any signed-in user, live immediately, logged.
#[patch("/api/store/{id}", session: tower_sessions::Session)]
pub async fn update_store(id: i64, fields: StoreFields) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    let before = db::store::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if before.deleted {
        return Err(AppError::deleted());
    }
    let write = parse_store(&fields)?;
    let after = db::store::update(pool(), id, &write, user.id).await?;
    db::edit_log::write(
        pool(),
        "store",
        id,
        EditAction::Update,
        &db::store::snapshot(&before),
        Some(&db::store::snapshot(&after)),
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, store_id = %id, "store updated");
    Ok(())
}

#[delete("/api/store/{id}", session: tower_sessions::Session)]
pub async fn delete_store(id: i64) -> ApiResult<()> {
    let user = auth::require_user(&session).await?;
    let before = db::store::find(pool(), id).await?.ok_or_else(AppError::not_found)?;
    if before.deleted {
        return Err(AppError::deleted());
    }
    db::store::soft_delete(pool(), id, user.id).await?;
    db::edit_log::write(pool(), "store", id, EditAction::Delete, &db::store::snapshot(&before), None, user.id)
        .await?;
    tracing::info!(user_id = %user.id, store_id = %id, "store deleted");
    Ok(())
}

/// Rate the shop 1–5 stars; rating again replaces the earlier one.
#[post("/api/store/{id}/review", session: tower_sessions::Session)]
pub async fn review_store(id: i64, stars: i16) -> ApiResult<ReviewSummary> {
    let user = auth::require_user(&session).await?;
    if !(1..=5).contains(&stars) {
        return Err(AppError::invalid("error-stars-range"));
    }
    db::store::find_public(pool(), id).await?.ok_or_else(AppError::not_found)?;
    db::review::upsert(pool(), id, user.id, stars).await?;
    Ok(db::review::summary(pool(), id, Some(user.id)).await?)
}

#[delete("/api/store/{id}/review", session: tower_sessions::Session)]
pub async fn remove_review(id: i64) -> ApiResult<ReviewSummary> {
    let user = auth::require_user(&session).await?;
    db::review::delete_own(pool(), id, user.id).await?;
    Ok(db::review::summary(pool(), id, Some(user.id)).await?)
}
