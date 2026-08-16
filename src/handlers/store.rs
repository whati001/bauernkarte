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
    handlers::product::{self, resolve_product},
    handlers::store_detail::load_detail_or_404,
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    models::{Category, Company, Product},
    opening_hours::{self, OpeningHoursFields},
    seasonality,
    sse::{patch_elements_at, patch_signals},
    state::AppState,
    templates::render,
};

#[derive(Template)]
#[template(path = "partials/store_form.html")]
struct StoreFormTemplate {
    is_edit: bool,
    action: String,
    /// Where the form's back button returns to
    /// (`handlers::back_action`): the store being edited, or the search
    /// panel for a brand-new one.
    back_action: String,
    name: String,
    /// `None` for a brand-new store — no position exists until the user
    /// clicks the map (see `static/map.js`'s location-picker module);
    /// `Some` pre-fills the picker at the store's current position when
    /// editing, so adjusting it is a drag rather than starting blank.
    lat: Option<f64>,
    lon: Option<f64>,
    opening_hours: Vec<opening_hours::WeekdayRow>,
    /// Every half hour, `"00:00"` .. `"24:00"` — the same list for all 14
    /// `<select>`s in the grid (`opening_hours::time_options`).
    time_options: Vec<String>,
    companies: Vec<Company>,
    /// Only populated (and only rendered, per the template's `!is_edit`
    /// guard) for the new-store form — a store SHALL carry at least one
    /// product on creation, see `handlers::product::resolve_product`'s
    /// doc comment for why. Editing an existing store doesn't touch its
    /// product list, so these stay empty there.
    products: Vec<Product>,
    categories: Vec<Category>,
    /// Same `!is_edit`-only scope as `products` above — the new-store
    /// form's repeating product blocks, revealed one at a time as the
    /// previous one is filled (`product::slot_views`).
    product_slots: Vec<product::ProductSlotView>,
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
        back_action: super::back_action(None),
        name: String::new(),
        lat: None,
        lon: None,
        opening_hours: opening_hours::week_rows(&[]),
        time_options: opening_hours::time_options(),
        companies,
        products,
        categories,
        product_slots: product::slot_views(),
        location_status_expr: location_status_expr(),
    });
    // Elements first, then signals — see `edit_form` below for why the
    // signal patch is needed at all.
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &html)),
        Ok(patch_signals(&opening_hours::form_signals(&[]))),
        Ok(patch_signals(&product::slot_signals())),
    ])))
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
    let hours = store.openinghours.map(|j| j.0).unwrap_or_default();
    let html = render(StoreFormTemplate {
        is_edit: true,
        action: format!("/store/{store_id}"),
        back_action: super::back_action(Some(store_id)),
        name: store.name,
        lat: Some(store.lat),
        lon: Some(store.lon),
        opening_hours: opening_hours::week_rows(&hours),
        time_options: opening_hours::time_options(),
        companies,
        products: Vec::new(),
        categories: Vec::new(),
        product_slots: Vec::new(),
        location_status_expr: location_status_expr(),
    });
    // The 14 `oh*` signals (and `hasOpeningHours`) outlive every #sidebar
    // swap, so without this the store edited *before* this one would
    // still be in them: `data-bind` seeds a signal from the element only
    // when it's missing, then drives the element from the signal, so a
    // stale value beats the freshly rendered `selected` option.
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &html)),
        Ok(patch_signals(&opening_hours::form_signals(&hours))),
    ])))
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
    company_id: Option<String>,
    #[serde(default)]
    is_company: bool,
    #[serde(default)]
    company_description: Option<String>,
    #[serde(default)]
    company_homepage: Option<String>,
    #[serde(flatten)]
    opening_hours: OpeningHoursFields,
    /// Everything else in the signal blob — the index-suffixed product
    /// blocks (`productId0`, `isSeasonal1`, …), unpacked by
    /// `product::parse_slots`. A catch-all rather than 5 x 18 named
    /// fields, and it keeps the slot shape defined in exactly one place.
    #[serde(flatten)]
    product_slots: std::collections::HashMap<String, serde_json::Value>,
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

    // All parsed/validated before any mutation (company/store/product) —
    // same "fail before committing" reasoning as `resolve_product` below.
    let hours = opening_hours::parse(&body.opening_hours)?;
    let slots = product::parse_slots(&body.product_slots)?;
    if slots.is_empty() {
        return Err(AppError::Validation("Bitte mindestens ein Produkt angeben.".into()));
    }
    // Each slot's seasonality is validated up front, before anything is
    // written — a bad month set in the *third* product must not leave a
    // company, a store and two listings behind.
    let mut resolved = Vec::with_capacity(slots.len());
    for slot in &slots {
        resolved.push(seasonality::parse(&slot.seasonality)?);
    }
    // Products are resolved (and, for brand-new ones, inserted) before
    // the company/store themselves — a store must carry at least one
    // product (see `resolve_product`'s doc comment), so failing this
    // validation must not leave an orphaned company/store behind.
    let mut products = Vec::with_capacity(slots.len());
    for slot in &slots {
        products.push(resolve_product(&state.pool, &slot.selection, user.id).await?);
    }

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
        (!hours.is_empty()).then_some(hours),
        user.id,
    )
    .await?;

    for (product, seasonal_months) in products.iter().zip(resolved) {
        db::store_product::insert(&state.pool, store.id, product.id, seasonal_months, user.id).await?;
    }
    tracing::info!(
        user_id = %user.id, store_id = %store.id, company_id = %company_id,
        product_count = products.len(), "store submitted for review"
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
    #[serde(flatten)]
    opening_hours: OpeningHoursFields,
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

    let hours = opening_hours::parse(&body.opening_hours)?;
    let old_snapshot = db::store::snapshot(&before);
    let after = db::store::update(
        &state.pool,
        store_id,
        before.company,
        name,
        body.store_lat,
        body.store_lon,
        (!hours.is_empty()).then_some(hours),
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
    let sidebar_html = crate::handlers::search::render_search_panel(&state, None, &results).await?;
    let map_data_html = crate::handlers::search::render_map_data(&results);
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &sidebar_html)),
        Ok(crate::sse::patch_elements(&map_data_html)),
    ])))
}
