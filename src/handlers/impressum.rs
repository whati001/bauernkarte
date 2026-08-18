//! The Impressum: a full page at `/impressum` and the same content as an
//! SSE panel at `/api/impressum`, which is what the sidebar footer link
//! swaps in. Same split as store detail (`/store/{id}` vs
//! `/api/store/{id}`) and for the same reason — a legal page needs a real
//! URL that can be linked and bookmarked, while a click inside the app
//! shouldn't throw away the map viewport.

use std::convert::Infallible;

use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Response, Sse},
};
use futures_util::stream;

use crate::{
    auth::OptionalUser,
    db,
    error::AppResult,
    i18n,
    i18n as filters, // see templates.rs's comment on this alias
    sse::patch_elements_at,
    state::AppState,
    templates::{full_page, render},
};

use askama::Template;

#[derive(Template)]
#[template(path = "partials/impressum.html")]
struct ImpressumTemplate {
    info: db::site_info::SiteInfo,
}

async fn render_panel(state: &AppState) -> AppResult<String> {
    let info = db::site_info::get(&state.pool).await?;
    Ok(render(ImpressumTemplate { info }))
}

/// `GET /impressum` — the linkable page.
pub async fn page(
    State(state): State<AppState>,
    OptionalUser(user): OptionalUser,
) -> AppResult<Response> {
    let sidebar_html = render_panel(&state).await?;
    // Unfiltered pins, same as a `/store/{id}` deep link: the map behind
    // the panel still needs something on it.
    let map_stores = db::store::search(&state.pool, None, None, None).await?;
    let map_data_html = crate::handlers::search::render_map_data(&map_stores);
    let signals = crate::handlers::pages::base_signals(
        crate::handlers::search::AUSTRIA_LAT,
        crate::handlers::search::AUSTRIA_LON,
        None,
        user.is_some(),
    );
    let nav_products =
        db::product::list_top_rated(&state.pool, crate::handlers::pages::NAV_PRODUCT_LIMIT).await?;
    let title = i18n::translate(i18n::current_locale(), "impressum-heading");
    Ok(full_page(
        &title,
        user.as_ref(),
        &signals,
        sidebar_html,
        map_data_html,
        false,
        nav_products,
    )
    .into_response())
}

/// `GET /api/impressum` — the panel, for the footer link.
pub async fn panel(
    State(state): State<AppState>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let html = render_panel(&state).await?;
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at(
        "#sidebar", "inner", &html,
    ))])))
}
