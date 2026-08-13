//! catalog-editing capability for `company`. (Companies are only ever
//! created as a side effect of `POST /store/new` — see `handlers::store`
//! — so this module is edit/delete only, no standalone create route.)

use askama::Template;
use axum::{
    extract::{Path, Query, State},
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
    handlers::store_detail::load_detail_or_404,
    i18n as filters, // see templates.rs's comment on this alias
    sse::patch_elements_at,
    state::AppState,
    templates::render,
};

#[derive(Template)]
#[template(path = "partials/company_form.html")]
struct CompanyFormTemplate {
    company_id: i64,
    store_id: Option<i64>,
    name: String,
    description: Option<String>,
    homepage: Option<String>,
}

#[derive(Deserialize)]
pub struct EditFormQuery {
    #[serde(default)]
    store_id: Option<i64>,
}

/// `GET /company/{id}/edit?store_id=` — `store_id` is carried through
/// purely so "save" can return to that store's detail view; company
/// itself isn't scoped to a store.
///
/// A plain `axum::extract::Query`, not `DatastarSignals` — this is a
/// literal `?store_id=` baked into the link's own `@get(...)` string at
/// render time (`sidebar_detail.html`), not a bound signal, so it's a
/// normal query param sitting alongside whatever `?datastar=...` blob
/// Datastar's own GET-action handling appends to the same URL.
pub async fn edit_form(
    State(state): State<AppState>,
    Path(company_id): Path<i64>,
    Query(q): Query<EditFormQuery>,
    CurrentUser(_user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let company = db::company::find(&state.pool, company_id).await?.ok_or(AppError::NotFound)?;
    if company.deleted {
        return Err(AppError::Conflict("company is deleted".into()));
    }
    let html = render(CompanyFormTemplate {
        company_id,
        store_id: q.store_id,
        name: company.name,
        description: company.description,
        homepage: company.homepage,
    });
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditCompanyBody {
    company_name: String,
    #[serde(default)]
    company_description: Option<String>,
    #[serde(default)]
    company_homepage: Option<String>,
    #[serde(default, deserialize_with = "crate::de::flexible_i64_opt")]
    return_store_id: Option<i64>,
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

/// `PATCH /company/{id}` (catalog-editing).
pub async fn update(
    State(state): State<AppState>,
    Path(company_id): Path<i64>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<EditCompanyBody>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::company::find(&state.pool, company_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("company is deleted".into()));
    }
    let name = body.company_name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Bitte einen Namen angeben.".into()));
    }

    let old_snapshot = db::company::snapshot(&before);
    let after = db::company::update(
        &state.pool,
        company_id,
        name,
        non_empty(body.company_description).as_deref(),
        non_empty(body.company_homepage).as_deref(),
        user.id,
    )
    .await?;
    let new_snapshot = db::company::snapshot(&after);
    db::edit_log::write(
        &state.pool,
        "company",
        company_id,
        db::edit_log::EditAction::Update,
        &old_snapshot,
        Some(&new_snapshot),
        user.id,
    )
    .await?;

    render_return(&state, body.return_store_id, user.id).await
}

/// `DELETE /company/{id}` (catalog-editing, soft delete).
pub async fn delete(
    State(state): State<AppState>,
    Path(company_id): Path<i64>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let before = db::company::find(&state.pool, company_id).await?.ok_or(AppError::NotFound)?;
    if before.deleted {
        return Err(AppError::Conflict("company already deleted".into()));
    }
    let old_snapshot = db::company::snapshot(&before);
    db::company::soft_delete(&state.pool, company_id, user.id).await?;
    db::edit_log::write(
        &state.pool,
        "company",
        company_id,
        db::edit_log::EditAction::Delete,
        &old_snapshot,
        None,
        user.id,
    )
    .await?;

    render_return(&state, None, user.id).await
}

async fn render_return(
    state: &AppState,
    return_store_id: Option<i64>,
    viewer_id: i64,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>> + use<>>> {
    let html = match return_store_id {
        Some(store_id) => {
            match load_detail_or_404(state, store_id, Some(viewer_id)).await {
                Ok(detail) => crate::handlers::store_detail::render_detail_panel(&detail, true),
                Err(_) => search_panel(state).await?,
            }
        }
        None => search_panel(state).await?,
    };
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

async fn search_panel(state: &AppState) -> AppResult<String> {
    let q = crate::handlers::search::SearchQuery::default();
    let results = crate::handlers::search::run_search(state, &q).await?;
    crate::handlers::search::render_search_panel(state, None, None, &results).await
}
