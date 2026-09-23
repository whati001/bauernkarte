//! The search panel: product filter and the matching stores, nearest
//! first once there's a geolocation fix.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use crate::{
    app::Route,
    components::{map::use_map, shell::use_catalog},
    i18n::use_locale,
    models::StoreSearchResult,
    ui::{
        badge::{Badge, BadgeVariant},
        item::{Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemTitle, ItemVariant},
        label::Label,
        select::{Select, SelectOption},
        skeleton::Skeleton,
    },
};

#[component]
pub fn SearchPanel() -> Element {
    let locale = use_locale();
    let map = use_map();

    rsx! {
        document::Title { {locale.t("nav-brand")} }
        div { class: "panel",
            ProductFilter {}
            match (map.results)() {
                None => rsx! {
                    for _ in 0..5 {
                        Skeleton { style: "height: 64px; margin-bottom: 8px; border-radius: 10px;" }
                    }
                },
                Some(results) => rsx! {
                    p { class: "result-count", {locale.t_count("search-results-count", results.len() as i64)} }
                    if results.is_empty() {
                        p { class: "muted", {locale.t("search-no-results")} }
                    }
                    ItemGroup { class: "results",
                        for store in results {
                            ResultItem { key: "{store.id}", store }
                        }
                    }
                },
            }
        }
    }
}

/// The same filter as the navbar picker and the chips — they all write
/// `MapCtx::product`.
#[component]
fn ProductFilter() -> Element {
    let locale = use_locale();
    let mut map = use_map();
    let catalog = use_catalog();
    let value = use_memo(move || Some((map.product)()));

    rsx! {
        div { class: "filter",
            Label { html_for: "product-filter",
                lucide::Package { size: 16 }
                {locale.t("search-product")}
            }
            Select::<Option<i64>> {
                id: "product-filter",
                value: Some(value.into()),
                on_value_change: move |v: Option<Option<i64>>| map.product.set(v.flatten()),
                SelectOption::<Option<i64>> { index: 0usize, value: None, text_value: locale.t("search-all"),
                    {locale.t("search-all")}
                }
                for (i , p) in catalog().into_iter().enumerate() {
                    SelectOption::<Option<i64>> {
                        key: "{p.id}",
                        index: i + 1,
                        value: Some(p.id),
                        text_value: format!("{} {}", p.icon_or_default(), p.name),
                        "{p.icon_or_default()} {p.name}"
                    }
                }
            }
        }
    }
}

#[component]
fn ResultItem(store: StoreSearchResult) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let id = store.id;
    let more = (store.product_total - store.products.len() as i64).max(0);

    rsx! {
        Item {
            class: "result",
            variant: ItemVariant::Outline,
            role: "button",
            tabindex: 0,
            onclick: move |_| {
                nav.push(Route::StorePanel { id });
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                    nav.push(Route::StorePanel { id });
                }
            },
            ItemContent {
                ItemTitle { "{store.name}" }
                ItemDescription { class: "result-products",
                    for p in store.products.iter() {
                        Badge { variant: BadgeVariant::Secondary,
                            "{p.icon.as_deref().unwrap_or(\"📦\")} {p.name}"
                            if p.rating_count > 0 {
                                " · {p.rating_count} ❤️"
                            }
                        }
                    }
                    if more > 0 {
                        Badge { variant: BadgeVariant::Outline, "+{more} {locale.t(\"search-more\")}" }
                    }
                }
            }
            if let Some(m) = store.distance_m {
                ItemActions { class: "result-distance",
                    lucide::MapPin { size: 14 }
                    {format!("{:.1} km", m / 1000.0)}
                }
            }
        }
    }
}
