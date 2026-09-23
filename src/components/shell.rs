//! The map layout: navbar on top, the sidebar (showing the current
//! route's panel) beside the map, and the floating map controls.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use crate::{
    api::search::{all_products, search_stores},
    app::Route,
    components::{
        common::{use_panel_data, PanelSkeleton},
        map::{MapCtx, MapView, Picker},
        navbar::Navbar,
    },
    i18n::use_locale,
    models::CatalogProduct,
    ui::button::{Button, ButtonSize, ButtonVariant},
};

/// The approved catalog, loaded once per visit: the navbar picker, the
/// filter select and the forms' product pickers all offer it.
#[derive(Clone, Copy)]
pub struct Catalog(pub Signal<Vec<CatalogProduct>>);

pub fn use_catalog() -> Signal<Vec<CatalogProduct>> {
    use_context::<Catalog>().0
}

/// The panel's width presets; the middle one is the default. Cycled by
/// the map's width button and remembered per browser.
const SIDEBAR_WIDTHS: [u32; 3] = [360, 420, 520];
const SIDEBAR_WIDTH_KEY: &str = "bk-sidebar-width";

#[component]
pub fn MapShell() -> Element {
    let route = use_route::<Route>();
    let locale = use_locale();

    // Map-first: the landing page starts with the panel closed; a deep
    // link to a store (or any form) starts with it open.
    let mut map = use_context_provider(|| MapCtx {
        results: Signal::new(None),
        product: Signal::new(None),
        geo: Signal::new(None),
        selected: Signal::new(None),
        picker: Signal::new(Picker::default()),
        sidebar_open: Signal::new(!matches!(route, Route::SearchPanel {})),
        refresh: Signal::new(0),
    });
    let mut catalog = use_context_provider(|| Catalog(Signal::new(Vec::new()))).0;

    use_resource(move || async move {
        if let Ok(products) = all_products().await {
            catalog.set(products);
        }
    });

    // One search feeds both the results list and the pins; it reruns
    // whenever the filter, the geolocation fix or a catalog change asks
    // for it. Loaded during server rendering too, so the first paint
    // doesn't wait for a round trip of its own.
    let product = (map.product)();
    let geo = (map.geo)();
    // Only a trigger: bumped after a catalog change so this reruns.
    let _refresh = (map.refresh)();
    let found = use_panel_data(use_reactive!(|(product, geo, _refresh)| search_stores(
        product,
        geo.map(|g| g.0),
        geo.map(|g| g.1)
    )))?;
    use_effect(use_reactive!(|found| {
        if let Some(Ok(results)) = found {
            map.results.set(Some(results));
        }
    }));

    // The panel's lifecycle follows the route: any panel other than the
    // search list opens it; leaving a store back to search closes it
    // again — unless the visitor opened it themselves.
    let mut opened_by_user = use_signal(|| false);
    let mut came_from_store = use_signal(|| false);
    let is_search = matches!(route, Route::SearchPanel {});
    let store_id = match route {
        Route::StorePanel { id } => Some(id),
        _ => None,
    };
    use_effect(use_reactive!(|is_search, store_id| {
        if !is_search {
            map.sidebar_open.set(true);
            if store_id.is_none() {
                opened_by_user.set(true);
            }
        } else if *came_from_store.peek() && !*opened_by_user.peek() {
            map.sidebar_open.set(false);
        }
        came_from_store.set(store_id.is_some());
        map.selected.set(store_id);
    }));

    // Escape closes an open store, like its back button. Skipped while
    // typing, and while a menu, list or dialog is open — Escape closes
    // that first, and only the next one leaves the store.
    let router = router();
    let nav = navigator();
    use_effect(move || {
        spawn(async move {
            let mut escapes = document::eval(
                r#"
                window.addEventListener("keydown", (e) => {
                    if (e.key !== "Escape" || e.defaultPrevented) return;
                    const t = e.target;
                    if (t.closest?.("input, textarea, select, [contenteditable], [role=menu], [role=listbox], [role=dialog], [role=alertdialog]")) return;
                    dioxus.send(true);
                });
                await new Promise(() => {});
                "#,
            );
            while escapes.recv::<bool>().await.is_ok() {
                if matches!(router.current::<Route>(), Route::StorePanel { .. }) {
                    nav.push(Route::SearchPanel {});
                }
            }
        });
    });

    let mut width = use_signal(|| SIDEBAR_WIDTHS[1]);
    use_effect(move || {
        spawn(async move {
            let saved = document::eval(&format!(
                "try {{ return Number(localStorage.getItem('{SIDEBAR_WIDTH_KEY}')) || 0; }} catch {{ return 0; }}"
            ))
            .await;
            if let Some(px) = saved.ok().and_then(|v| v.as_u64()).map(|v| v as u32) {
                if SIDEBAR_WIDTHS.contains(&px) {
                    width.set(px);
                }
            }
        });
    });
    let cycle_width = move |_| {
        let next = SIDEBAR_WIDTHS[(SIDEBAR_WIDTHS.iter().position(|w| *w == width()).unwrap_or(1) + 1) % 3];
        width.set(next);
        document::eval(&format!("try {{ localStorage.setItem('{SIDEBAR_WIDTH_KEY}', '{next}'); }} catch {{}}"));
    };

    let open = (map.sidebar_open)();
    let mut locating = use_signal(|| false);
    let toggle_label = if open { locale.t("map-sidebar-collapse") } else { locale.t("map-sidebar-expand") };

    rsx! {
        div { class: "app",
            Navbar { with_search: true }
            div {
                id: "layout",
                class: if !open { "sidebar-collapsed" },
                style: "--sidebar-width: {width}px",
                div { id: "sidebar-column",
                    div { id: "sidebar",
                        SuspenseBoundary {
                            fallback: |_| rsx! { PanelSkeleton {} },
                            Outlet::<Route> {}
                        }
                    }
                    footer { id: "sidebar-footer",
                        Link { to: Route::Impressum {}, {locale.t("impressum-link")} }
                    }
                }
                Button {
                    id: "sidebar-open",
                    class: "map-fab",
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Icon,
                    title: "{toggle_label}",
                    "aria-label": "{toggle_label}",
                    onclick: move |_| {
                        let now_open = !(map.sidebar_open)();
                        opened_by_user.set(now_open);
                        map.sidebar_open.set(now_open);
                    },
                    lucide::Layers { size: 18 }
                }
                Button {
                    id: "map-locate",
                    class: if locating() { "map-fab locating" } else { "map-fab" },
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Icon,
                    title: locale.t("search-use-my-location"),
                    "aria-label": locale.t("search-use-my-location"),
                    onclick: move |_| async move {
                        locating.set(true);
                        let _ = document::eval("return await window.BK.locate();").await;
                        locating.set(false);
                    },
                    lucide::LocateFixed { size: 18 }
                }
                Button {
                    id: "sidebar-width",
                    class: "map-fab",
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Icon,
                    title: locale.t("map-sidebar-width"),
                    "aria-label": locale.t("map-sidebar-width"),
                    onclick: cycle_width,
                    lucide::MoveHorizontal { size: 18 }
                }
                MapView {}
            }
        }
    }
}
