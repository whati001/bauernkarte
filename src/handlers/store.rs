//! community-submissions capability (create) + catalog-editing capability
//! (edit/delete) for `store` and `company`.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream;
use serde::Deserialize;
use std::convert::Infallible;

use crate::{
    auth::CurrentUser,
    db,
    error::{AppError, AppResult},
    handlers::product::{resolve_product, NewProductSelection},
    handlers::store_detail::load_detail_or_404,
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    models::{Category, Company, Product},
    sse::patch_elements_at,
    state::AppState,
    templates::render,
};

#[derive(Template)]
#[template(path = "partials/store_form.html")]
struct StoreFormTemplate {
    is_edit: bool,
    action: String,
    name: String,
    /// `None` for a brand-new store — no position exists until the user
    /// clicks the map (see `static/map.js`'s location-picker module);
    /// `Some` pre-fills the picker at the store's current position when
    /// editing, so adjusting it is a drag rather than starting blank.
    lat: Option<f64>,
    lon: Option<f64>,
    openinghours: Option<String>,
    companies: Vec<Company>,
    /// Only populated (and only rendered, per the template's `!is_edit`
    /// guard) for the new-store form — a store SHALL carry at least one
    /// product on creation, see `handlers::product::resolve_product`'s
    /// doc comment for why. Editing an existing store doesn't touch its
    /// product list, so these stay empty there.
    products: Vec<Product>,
    categories: Vec<Category>,
    /// Pre-built `data-text` JS ternary for the picker status line —
    /// built here (not in the template) so the two translated fragments
    /// go through `serde_json::to_string` for correct JS-string escaping
    /// rather than hand-rolled quoting in the `.html` file.
    location_status_expr: String,
}

fn location_status_expr() -> String {
    let locale = crate::i18n::current_locale();
    let picked = serde_json::to_string(&crate::i18n::translate(locale, "search-location-picked"))
        .unwrap_or_default();
    let prompt = serde_json::to_string(&crate::i18n::translate(locale, "search-pick-on-map"))
        .unwrap_or_default();
    format!("$storeLat && $storeLon ? {picked} : {prompt}")
}

/// `GET /store/new` — new-store form fragment (community-submissions).
pub async fn new_form(
    State(state): State<AppState>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let companies = db::company::list_approved(&state.pool).await?;
    // All approved products across every category — same flat-select
    // rationale as `product::new_form`.
    let products = db::product::list_all_approved(&state.pool).await?;
    let categories = db::category::list_all(&state.pool).await?;
    let html = render(StoreFormTemplate {
        is_edit: false,
        action: "/store/new".to_string(),
        name: String::new(),
        lat: None,
        lon: None,
        openinghours: None,
        companies,
        products,
        categories,
        location_status_expr: location_status_expr(),
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `GET /store/{id}/edit` — catalog-editing: any logged-in user.
pub async fn edit_form(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let store = db::store::find(&state.pool, store_id).await?.ok_or(AppError::NotFound)?;
    if store.deleted {
        return Err(AppError::Conflict("store is deleted".into()));
    }
    let companies = db::company::list_approved(&state.pool).await?;
    let html = render(StoreFormTemplate {
        is_edit: true,
        action: format!("/store/{store_id}"),
        name: store.name,
        lat: Some(store.lat),
        lon: Some(store.lon),
        openinghours: store.openinghours,
        companies,
        products: Vec::new(),
        categories: Vec::new(),
        location_status_expr: location_status_expr(),
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewStoreBody {
    store_name: String,
    #[serde(deserialize_with = "crate::de::flexible_f64")]
    store_lat: f64,
    #[serde(deserialize_with = "crate::de::flexible_f64")]
    store_lon: f64,
    #[serde(default)]
    store_openinghours: Option<String>,
    #[serde(default)]
    company_id: Option<String>,
    #[serde(default)]
    is_company: bool,
    #[serde(default)]
    company_description: Option<String>,
    #[serde(default)]
    company_homepage: Option<String>,
    #[serde(flatten)]
    product: NewProductSelection,
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

/// `POST /store/new` — creates `company` (maybe) + `store`, both
/// `approved=false` (community-submissions capability). Server-side
/// validation: exactly one of `{company_id, isCompany}`.
pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<NewStoreBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let name = body.store_name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Bitte einen Namen angeben.".into()));
    }

    let company_id = non_empty(body.company_id).and_then(|s| s.parse::<i64>().ok());
    let has_company_id = company_id.is_some();
    if has_company_id == body.is_company {
        return Err(AppError::Validation(
            "Bitte entweder eine bestehende Firma wählen ODER angeben, dass dieses Geschäft die Firma ist — nicht beides oder keines.".into(),
        ));
    }

    // Resolved (and, for a brand-new product, inserted) before the
    // company/store themselves — a store must carry at least one product
    // (see `resolve_product`'s doc comment), so failing this validation
    // must not leave an orphaned company/store behind.
    let product = resolve_product(&state.pool, &body.product, user.id).await?;

    let company_id = if body.is_company {
        let company = db::company::insert(
            &state.pool,
            name,
            non_empty(body.company_description).as_deref(),
            non_empty(body.company_homepage).as_deref(),
            user.id,
        )
        .await?;
        company.id
    } else {
        company_id.expect("validated above: exactly one of company_id/is_company")
    };

    let store = db::store::insert(
        &state.pool,
        company_id,
        name,
        body.store_lat,
        body.store_lon,
        non_empty(body.store_openinghours).as_deref(),
        user.id,
    )
    .await?;

    db::store_product::insert(&state.pool, store.id, product.id, user.id).await?;
    tracing::info!(
        user_id = %user.id, store_id = %store.id, company_id = %company_id, product_id = %product.id,
        "store submitted for review"
    );

    let html = crate::templates::render_confirmation(&i18n::translate_with_name(
        i18n::current_locale(),
        "confirmation-pending",
        &store.name,
    ));
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditStoreBody {
    store_name: String,
    #[serde(deserialize_with = "crate::de::flexible_f64")]
    store_lat: f64,
    #[serde(deserialize_with = "crate::de::flexible_f64")]
    store_lon: f64,
    #[serde(default)]
    store_openinghours: Option<String>,
}

/// `PATCH /store/{id}` — catalog-editing: any logged-in user, live
/// immediately, `approved` untouched, logged to `edit_log`.
pub async fn update(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<EditStoreBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::store::find(&state.pool, store_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("store is deleted".into()));
    }
    let name = body.store_name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Bitte einen Namen angeben.".into()));
    }

    let old_snapshot = db::store::snapshot(&before);
    let after = db::store::update(
        &state.pool,
        store_id,
        before.company,
        name,
        body.store_lat,
        body.store_lon,
        non_empty(body.store_openinghours).as_deref(),
        user.id,
    )
    .await?;
    let new_snapshot = db::store::snapshot(&after);
    db::edit_log::write(
        &state.pool,
        "store",
        store_id,
        db::edit_log::EditAction::Update,
        &old_snapshot,
        Some(&new_snapshot),
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, store_id = %store_id, "store updated");

    let detail = load_detail_or_404(&state, store_id, Some(user.id)).await?;
    let html = crate::handlers::store_detail::render_detail_panel(&detail, true);
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `DELETE /store/{id}` — catalog-editing: soft delete, any logged-in
/// user, logged to `edit_log`.
pub async fn delete(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::store::find(&state.pool, store_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("store already deleted".into()));
    }
    let old_snapshot = db::store::snapshot(&before);
    db::store::soft_delete(&state.pool, store_id, user.id).await?;
    db::edit_log::write(
        &state.pool,
        "store",
        store_id,
        db::edit_log::EditAction::Delete,
        &old_snapshot,
        None,
        user.id,
    )
    .await?;
    tracing::info!(user_id = %user.id, store_id = %store_id, "store deleted");

    // The store the visitor was looking at is gone — send them back to
    // search rather than a now-404ing detail view.
    let q = crate::handlers::search::SearchQuery::default();
    let results = crate::handlers::search::run_search(&state, &q).await?;
    let panel = crate::handlers::search::render_search_panel(&state, None, None, &results).await?;
    let map_data_html = crate::handlers::search::render_map_data(&panel.map_stores);
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &panel.sidebar_html)),
        Ok(crate::sse::patch_elements(&map_data_html)),
    ])))
}
