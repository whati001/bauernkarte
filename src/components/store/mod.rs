//! The store info panel: a header image with the shop's name, the key
//! facts (what it sells, rating, address, hours), who runs it, and its
//! products and when they're in season — then photos and the actions.

mod facts;
mod gallery;
mod hero;
mod manage;
mod offers;
mod owner;

use dioxus::prelude::*;

use crate::{
    api::store::store_detail,
    components::common::{use_panel_data, LoadError, PanelSkeleton},
    models::StoreDetail,
    ui::separator::Separator,
};

#[css_module("/src/components/store/style.css")]
pub(crate) struct Styles;

/// The panel's data, shared by its parts. A local copy of what the server
/// sent, updated in place after the visitor changes something (a heart, a
/// rating) so the panel never flashes back to a loading state.
#[derive(Clone, Copy)]
pub struct StoreState {
    pub detail: Signal<StoreDetail>,
    /// The photo open in the full-screen viewer, by index into `photos`.
    pub viewing: Signal<Option<usize>>,
}

impl StoreState {
    pub fn reload(self) {
        let mut detail = self.detail;
        let id = detail.peek().id;
        spawn(async move {
            if let Ok(fresh) = store_detail(id).await {
                detail.set(fresh);
            }
        });
    }
}

pub fn use_store() -> StoreState {
    use_context::<StoreState>()
}

pub fn directions_url(lat: f64, lon: f64) -> String {
    format!("https://www.google.com/maps/dir/?api=1&destination={lat},{lon}")
}

/// Server-rendered on a deep link (`/store/{id}`), so a shared link shows
/// the shop before any WASM has loaded.
///
/// One data hook whose future re-runs when `id` changes, and which never
/// suspends in the browser (see `use_panel_data`) — so picking another
/// store while one is open actually swaps the panel.
#[component]
pub fn StorePanel(id: i64) -> Element {
    let detail = use_panel_data(use_reactive!(|id| store_detail(id)))?;
    match detail {
        // Keyed, so the panel's local copy starts over for each store.
        Some(Ok(detail)) => rsx! { StoreView { key: "{detail.id}", detail } },
        Some(Err(error)) => rsx! { LoadError { error } },
        None => rsx! { PanelSkeleton {} },
    }
}

#[component]
fn StoreView(detail: StoreDetail) -> Element {
    let detail = use_signal(|| detail);
    let viewing = use_signal(|| None);
    use_context_provider(|| StoreState { detail, viewing });
    let d = detail.read();

    rsx! {
        document::Title { "{d.name} – BauernKarte" }
        article { class: Styles::store, "data-store-id": d.id,
            hero::StoreHero {}
            div { class: Styles::store_body,
                facts::StoreFacts {}
                if d.owner_name.is_some() {
                    Separator { class: Styles::rule, horizontal: true, decorative: true }
                    owner::OwnerSection {}
                }
                Separator { class: Styles::rule, horizontal: true, decorative: true }
                offers::OfferList {}
                if !d.photos.is_empty() {
                    manage::PhotoStrip {}
                }
                manage::StoreActions {}
            }
            gallery::PhotoViewer {}
        }
    }
}
