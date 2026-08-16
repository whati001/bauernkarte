//! store-detail capability: `GET /api/store/{id}`, `GET /api/store/back`,
//! and the full-page `GET /store/{id}` deep link (handled in `pages.rs`,
//! reusing `render_detail_panel` here).

use askama::Template;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
};
use futures_util::stream;
use std::convert::Infallible;

use crate::{
    auth::OptionalUser,
    db,
    dstar::DatastarSignals,
    error::{AppError, AppResult},
    handlers::search::{render_map_data, render_search_panel, run_search, SearchQuery},
    i18n as filters, // see templates.rs's comment on this alias
    models::{RatingCount, SiblingStore, StoreDetail},
    opening_hours,
    seasonality,
    sse::{patch_elements, patch_elements_at},
    state::AppState,
    templates::render,
};

struct ImageView {
    id: i64,
    /// The uploader's caption when there is one, otherwise the product
    /// the photo belongs to — the carousel is store-wide now, so a photo
    /// with no caption still says what it's of.
    caption: String,
}

/// Foliage hue band for the header illustration: yellow-green through
/// green. Wide enough that different products look different, narrow
/// enough that every one of them still looks like farmland.
const HERO_HUE_BASE: i64 = 72;
const HERO_HUE_SPAN: i64 = 58;

/// The header illustration, used when a store has no uploaded photo.
///
/// It's drawn (an inline SVG farm scene, see `store_hero.html`), not a
/// stock photo, and it's keyed to the store's *lead product*: `hue`
/// comes from that product's id, so one product always yields the same
/// picture and two stores leading with the same product look alike on
/// purpose. `icons` are the store's product emoji laid into the crate in
/// the foreground.
///
/// `hue` is deliberately confined to `HERO_HUE_BASE .. +HERO_HUE_SPAN` —
/// a free 0..360 hue gave some products magenta fields, which reads as a
/// bug rather than as variety. Only the land varies; the sky is fixed.
struct HeroArt {
    hue: i64,
    icons: Vec<String>,
}

struct StoreProductView {
    store_product_id: i64,
    product_id: i64,
    product_name: String,
    product_description: Option<String>,
    product_icon: Option<String>,
    category_name: String,
    category_icon: Option<String>,
    /// Always 12 rows (`seasonality::month_rows`) — the store-detail
    /// month bar shows every month regardless of whether the listing
    /// restricts any of them.
    seasonal_months: Vec<seasonality::MonthRow>,
    /// "Jan..Jun, Sep..Dez"-style plain text, used both as the bar's
    /// `aria-label` and, in the spec grid, as its visible value — see
    /// `seasonality::season_summary`'s doc comment.
    season_summary: String,
    ratings: Vec<RatingCount>,
    viewer_has_rated_up: bool,
    /// Pre-pluralised "3 Fotos" for the spec grid's images cell — built
    /// here rather than in the template because the `|t` filter can't
    /// pass Fluent arguments (see `i18n::translate_with_count`).
    image_count_label: String,
    selected: bool,
}

#[derive(Template)]
#[template(path = "partials/sidebar_detail.html")]
struct SidebarDetailTemplate {
    store_id: i64,
    store_name: String,
    /// `Some` only when at least one day has hours specified — the
    /// section is hidden entirely otherwise, same as before this was
    /// structured (see `models::StoreDetail::openinghours`'s comment).
    opening_hours: Option<Vec<opening_hours::WeekdayRow>>,
    lat: f64,
    lon: f64,
    company_id: i64,
    company_name: String,
    company_description: Option<String>,
    company_homepage: Option<String>,
    products: Vec<StoreProductView>,
    /// Pre-pluralised "2 Produkte" for the store card's footer — see
    /// `StoreProductView::image_count_label` on why it isn't done in the
    /// template.
    product_count_label: String,
    /// The store's first uploaded photo, used as the header image.
    /// `None` falls back to `hero_art`.
    hero_image_id: Option<i64>,
    hero_art: HeroArt,
    /// Every approved photo across all of the store's products, for the
    /// carousel. Empty means the whole section is skipped.
    photos: Vec<ImageView>,
    sibling_stores: Vec<SiblingStore>,
    /// Prebuilt Google Maps link for the "Get directions" button.
    maps_url: String,
    logged_in: bool,
}

pub fn render_detail_panel(detail: &StoreDetail, logged_in: bool) -> String {
    render_detail_panel_with_selection(detail, logged_in, None)
}

pub fn render_detail_panel_with_selection(
    detail: &StoreDetail,
    logged_in: bool,
    selected_store_product_id: Option<i64>,
) -> String {
    let locale = crate::i18n::current_locale();
    let products: Vec<StoreProductView> = detail
        .products
        .iter()
        .map(|p| StoreProductView {
            store_product_id: p.store_product_id,
            product_id: p.product_id,
            product_name: p.product_name.clone(),
            product_description: p.product_description.clone(),
            product_icon: p.product_icon.clone(),
            category_name: p.category_name.clone(),
            category_icon: p.category_icon.clone(),
            seasonal_months: seasonality::month_rows(p.seasonal_months.as_deref()),
            season_summary: seasonality::season_summary(p.seasonal_months.as_deref()),
            ratings: p.ratings.clone(),
            viewer_has_rated_up: p.viewer_has_rated_up,
            image_count_label: crate::i18n::translate_with_count(
                locale,
                "detail-image-count",
                p.images.len() as i64,
            ),
            selected: selected_store_product_id == Some(p.store_product_id),
        })
        .collect();

    // Flattened across products: the carousel is a property of the store,
    // not of one listing, so a shop with one photo on each of three
    // products still gets a three-photo strip.
    let photos: Vec<ImageView> = detail
        .products
        .iter()
        .flat_map(|p| {
            p.images.iter().map(|i| ImageView {
                id: i.id,
                caption: i.description.clone().unwrap_or_else(|| p.product_name.clone()),
            })
        })
        .collect();

    let hero_art = HeroArt {
        // Products come back ordered by name, so "lead product" is stable
        // for a given store; the hue is derived from the product's own
        // id, which is what makes the illustration per-product rather
        // than per-store.
        hue: detail
            .products
            .first()
            .map(|p| HERO_HUE_BASE + (p.product_id * 37).rem_euclid(HERO_HUE_SPAN))
            .unwrap_or(HERO_HUE_BASE + 26),
        icons: detail
            .products
            .iter()
            .take(3)
            .map(|p| p.product_icon.clone().unwrap_or_else(|| "\u{1f33e}".to_string()))
            .collect(),
    };

    render(SidebarDetailTemplate {
        store_id: detail.store_id,
        store_name: detail.store_name.clone(),
        opening_hours: (!detail.openinghours.is_empty()).then(|| opening_hours::week_rows(&detail.openinghours)),
        lat: detail.lat,
        lon: detail.lon,
        company_id: detail.company_id,
        company_name: detail.company_name.clone(),
        company_description: detail.company_description.clone(),
        company_homepage: detail.company_homepage.clone(),
        product_count_label: crate::i18n::translate_with_count(
            locale,
            "detail-product-count",
            products.len() as i64,
        ),
        products,
        hero_image_id: photos.first().map(|i| i.id),
        hero_art,
        photos,
        sibling_stores: detail.sibling_stores.clone(),
        maps_url: format!(
            "https://www.google.com/maps/dir/?api=1&destination={},{}",
            detail.lat, detail.lon
        ),
        logged_in,
    })
}

pub async fn load_detail_or_404(state: &AppState, store_id: i64, viewer_id: Option<i64>) -> AppResult<StoreDetail> {
    db::detail::get_store_detail(&state.pool, store_id, viewer_id)
        .await?
        .ok_or(AppError::NotFound)
}

/// `GET /api/store/{id}` — `patch-elements #sidebar` (mode `inner`) with
/// the detail fragment (store-detail capability).
pub async fn show(
    State(state): State<AppState>,
    Path(store_id): Path<i64>,
    OptionalUser(user): OptionalUser,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let viewer_id = user.as_ref().map(|u| u.id);
    let detail = load_detail_or_404(&state, store_id, viewer_id).await?;
    tracing::debug!(store_id = %store_id, viewer_id = ?viewer_id, "store detail viewed");
    let html = render_detail_panel(&detail, user.is_some());
    Ok(Sse::new(stream::iter(vec![Ok(patch_elements_at("#sidebar", "inner", &html))])))
}

/// `GET /api/store/back` — re-renders the search panel using whatever
/// filter/location signals the client currently holds (Datastar sends
/// all current signals as query params on a GET action by default, so
/// this reflects the exact prior search without the server tracking any
/// "previous state" itself).
pub async fn back(
    State(state): State<AppState>,
    DatastarSignals(q): DatastarSignals<SearchQuery>,
) -> AppResult<Sse<impl stream::Stream<Item = Result<Event, Infallible>>>> {
    let results = run_search(&state, &q).await?;
    let sidebar_html = render_search_panel(&state, q.category_id, &results).await?;
    let map_data_html = render_map_data(&results);
    Ok(Sse::new(stream::iter(vec![
        Ok(patch_elements_at("#sidebar", "inner", &sidebar_html)),
        Ok(patch_elements(&map_data_html)),
    ])))
}
