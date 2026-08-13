//! community-submissions capability (add product to store) + catalog-editing
//! for `product` and `store_product`.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::convert::Infallible;

use crate::{
    auth::CurrentUser,
    db,
    error::{AppError, AppResult},
    handlers::store_detail::load_detail_or_404,
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    models::{Category, Product},
    sse::patch_elements_at,
    state::AppState,
    templates::{render, render_confirmation},
};

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

async fn detail_html(state: &AppState, store_id: i64, viewer_id: i64) -> AppResult<String> {
    let detail = load_detail_or_404(state, store_id, Some(viewer_id)).await?;
    Ok(crate::handlers::store_detail::render_detail_panel(&detail, true))
}

// ---- add product to store (create) ----

#[derive(Template)]
#[template(path = "partials/product_form.html")]
struct ProductFormTemplate {
    store_id: i64,
    /// Pre-translated "Add product to {store}" heading — built here (not
    /// in the template) since the `t` filter only takes a literal key, no
    /// interpolation args; mirrors how `confirmation-pending` etc. are
    /// resolved via `i18n::translate_with_name` elsewhere.
    heading: String,
    products: Vec<Product>,
    categories: Vec<Category>,
}

/// `GET /store/{id}/product/new` (community-submissions).
pub async fn new_form(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let store = db::store::find_public(&state.pool, store_id).await?.ok_or(AppError::NotFound)?;
    let categories = db::category::list_all(&state.pool).await?;
    // All approved products across every category — a farm-shop catalog
    // is small enough in v1 that a single flat select is fine (no
    // category-cascade needed here, unlike the search filter).
    let mut products = Vec::new();
    for c in &categories {
        products.extend(db::product::list_approved_by_category(&state.pool, c.id).await?);
    }
    let heading = i18n::translate_with_name(i18n::current_locale(), "product-form-add-heading", &store.name);
    let html = render(ProductFormTemplate {
        store_id,
        heading,
        products,
        categories,
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewStoreProductBody {
    #[serde(default)]
    is_new_product: bool,
    #[serde(default)]
    product_id: Option<String>,
    #[serde(default)]
    new_product_name: Option<String>,
    #[serde(default)]
    new_product_category_id: Option<String>,
    #[serde(default)]
    new_product_description: Option<String>,
    price: Decimal,
}

/// `POST /store/{id}/product/new` — creates `product` (maybe,
/// `approved=false`) + `store_product` (`approved=false`)
/// (community-submissions capability).
pub async fn create(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<NewStoreProductBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    if db::store::find_public(&state.pool, store_id).await?.is_none() {
        return Err(AppError::NotFound);
    }
    if body.price < Decimal::ZERO {
        return Err(AppError::Validation("Der Preis darf nicht negativ sein.".into()));
    }

    let product = if body.is_new_product {
        let name = non_empty(body.new_product_name).ok_or_else(|| {
            AppError::Validation("Bitte einen Produktnamen angeben.".into())
        })?;
        let category_id: i64 = non_empty(body.new_product_category_id)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| AppError::Validation("Bitte eine Kategorie wählen.".into()))?;
        if !db::category::exists(&state.pool, category_id).await? {
            return Err(AppError::Validation("Unbekannte Kategorie.".into()));
        }
        db::product::insert(
            &state.pool,
            category_id,
            &name,
            non_empty(body.new_product_description).as_deref(),
            user.id,
        )
        .await?
    } else {
        let product_id: i64 = non_empty(body.product_id)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| AppError::Validation("Bitte ein Produkt wählen.".into()))?;
        db::product::find(&state.pool, product_id).await?.ok_or(AppError::NotFound)?
    };

    let store_product = db::store_product::insert(&state.pool, store_id, product.id, body.price, user.id).await?;
    let _ = store_product; // id not needed beyond confirming creation succeeded

    let html = render_confirmation(&i18n::translate_with_name(
        i18n::current_locale(),
        "confirmation-pending",
        &product.name,
    ));
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

// ---- store_product edit/delete (price) ----

#[derive(Template)]
#[template(path = "partials/store_product_form.html")]
struct StoreProductFormTemplate {
    store_product_id: i64,
    /// Pre-translated "Edit price: {product}" heading — see
    /// `ProductFormTemplate::heading`'s comment for why this isn't built
    /// in the template.
    heading: String,
    price: String,
}

/// `GET /store-product/{id}/edit` (catalog-editing).
pub async fn edit_form(
    State(state): State<AppState>,
    Path(store_product_id): Path<i64>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let sp = db::store_product::find(&state.pool, store_product_id).await?.ok_or(AppError::NotFound)?;
    if sp.deleted {
        return Err(AppError::Conflict("listing is deleted".into()));
    }
    let product = db::product::find(&state.pool, sp.product).await?.ok_or(AppError::NotFound)?;
    let heading = i18n::translate_with_name(i18n::current_locale(), "store-product-form-heading", &product.name);
    let html = render(StoreProductFormTemplate {
        store_product_id,
        heading,
        price: format!("{:.2}", sp.price),
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

#[derive(Deserialize)]
pub struct EditStoreProductBody {
    price: Decimal,
}

/// `PATCH /store-product/{id}` (catalog-editing).
pub async fn update(
    State(state): State<AppState>,
    Path(store_product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<EditStoreProductBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::store_product::find(&state.pool, store_product_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("listing is deleted".into()));
    }
    if body.price < Decimal::ZERO {
        return Err(AppError::Validation("Der Preis darf nicht negativ sein.".into()));
    }

    let old_snapshot = db::store_product::snapshot(&before);
    let after = db::store_product::update(&state.pool, store_product_id, body.price, user.id).await?;
    let new_snapshot = db::store_product::snapshot(&after);
    db::edit_log::write(
        &state.pool,
        "store_product",
        store_product_id,
        db::edit_log::EditAction::Update,
        &old_snapshot,
        Some(&new_snapshot),
        user.id,
    )
    .await?;

    let html = detail_html(&state, before.store, user.id).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `DELETE /store-product/{id}` (catalog-editing, soft delete).
pub async fn delete(
    State(state): State<AppState>,
    Path(store_product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::store_product::find(&state.pool, store_product_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("listing already deleted".into()));
    }
    let old_snapshot = db::store_product::snapshot(&before);
    db::store_product::soft_delete(&state.pool, store_product_id, user.id).await?;
    db::edit_log::write(
        &state.pool,
        "store_product",
        store_product_id,
        db::edit_log::EditAction::Delete,
        &old_snapshot,
        None,
        user.id,
    )
    .await?;

    let html = detail_html(&state, before.store, user.id).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

// ---- product entity edit/delete ----

#[derive(Template)]
#[template(path = "partials/edit_product_form.html")]
struct EditProductFormTemplate {
    product_id: i64,
    name: String,
    description: Option<String>,
    category_id: i64,
    categories: Vec<Category>,
}

/// `GET /product/{id}/edit` (catalog-editing).
pub async fn edit_product_form(
    State(state): State<AppState>,
    Path(product_id): Path<i64>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let product = db::product::find(&state.pool, product_id).await?.ok_or(AppError::NotFound)?;
    if product.deleted {
        return Err(AppError::Conflict("product is deleted".into()));
    }
    let categories = db::category::list_all(&state.pool).await?;
    let html = render(EditProductFormTemplate {
        product_id,
        name: product.name,
        description: product.description,
        category_id: product.category,
        categories,
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditProductBody {
    product_name: String,
    #[serde(deserialize_with = "crate::de::flexible_i64")]
    product_category_id: i64,
    #[serde(default)]
    product_description: Option<String>,
}

/// `PATCH /product/{id}` (catalog-editing).
pub async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<EditProductBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::product::find(&state.pool, product_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("product is deleted".into()));
    }
    let name = body.product_name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Bitte einen Namen angeben.".into()));
    }
    if !db::category::exists(&state.pool, body.product_category_id).await? {
        return Err(AppError::Validation("Unbekannte Kategorie.".into()));
    }

    let old_snapshot = db::product::snapshot(&before);
    let after = db::product::update(
        &state.pool,
        product_id,
        body.product_category_id,
        name,
        non_empty(body.product_description).as_deref(),
        user.id,
    )
    .await?;
    let new_snapshot = db::product::snapshot(&after);
    db::edit_log::write(
        &state.pool,
        "product",
        product_id,
        db::edit_log::EditAction::Update,
        &old_snapshot,
        Some(&new_snapshot),
        user.id,
    )
    .await?;

    // Unlike the creation confirmation above, this is catalog-editing, not
    // community-submissions — the edit is already live (no moderation
    // gate), so the message says so instead of the misleading "wird
    // geprüft" wording a copy-pasted call here previously used.
    let html = render_confirmation(&i18n::translate_with_name(
        i18n::current_locale(),
        "confirmation-updated",
        &after.name,
    ));
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `DELETE /product/{id}` (catalog-editing, soft delete). Note this
/// removes the product from the shared taxonomy entirely (not just one
/// store's listing of it) — that's `DELETE /store-product/{id}` instead.
pub async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::product::find(&state.pool, product_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("product already deleted".into()));
    }
    let old_snapshot = db::product::snapshot(&before);
    db::product::soft_delete(&state.pool, product_id, user.id).await?;
    db::edit_log::write(
        &state.pool,
        "product",
        product_id,
        db::edit_log::EditAction::Delete,
        &old_snapshot,
        None,
        user.id,
    )
    .await?;

    let q = crate::handlers::search::SearchQuery::default();
    let results = crate::handlers::search::run_search(&state, &q).await?;
    let html = crate::handlers::search::render_search_panel(&state, None, None, &results).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}
