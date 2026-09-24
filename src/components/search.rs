//! The search panel: product, type and sort controls over the matching
//! stores, each a card with its photo, products, town and whether it's
//! open today — nearest first once there's a geolocation fix.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use crate::{
    app::Route,
    components::{map::use_map, shell::use_catalog},
    i18n::use_locale,
    models::{StoreKind, StoreSearchResult},
    opening_hours::{self, TodayStatus},
    ui::{
        badge::{Badge, BadgeVariant},
        input::Input,
        select::{Select, SelectOption},
        skeleton::Skeleton,
    },
};

/// How many product chips a card shows before "+N".
const CHIP_LIMIT: usize = 3;

#[derive(Clone, Copy, PartialEq)]
enum Sort {
    /// The server's order: by distance with a location fix, else by name.
    Distance,
    Name,
}

#[component]
pub fn SearchPanel() -> Element {
    let locale = use_locale();
    let map = use_map();
    let sort = use_signal(|| Sort::Distance);

    // The visitor's weekday and time, for "open today". Browser-only: the
    // server's clock is in the wrong time zone, so the server-rendered
    // list leaves the status out and the browser fills it in.
    let mut now = use_signal(|| None::<(i16, String)>);
    use_effect(move || {
        spawn(async move {
            let value = document::eval(
                "const d = new Date(); \
                 return [d.getDay() || 7, `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`];",
            )
            .await;
            if let Some(parsed) = value.ok().and_then(|v| serde_json::from_value(v).ok()) {
                now.set(Some(parsed));
            }
        });
    });

    rsx! {
        document::Title { {locale.t("nav-brand")} }
        div { class: "panel",
            div { class: "search-toolbar",
                NameSearch {}
                ProductFilter {}
                KindFilter {}
                SortSelect { sort }
            }
            match (map.results)() {
                None => rsx! {
                    for _ in 0..5 {
                        Skeleton { style: "height: 96px; margin-bottom: 8px; border-radius: 10px;" }
                    }
                },
                Some(mut results) => {
                    if sort() == Sort::Name {
                        results.sort_by_key(|s| s.name.to_lowercase());
                    }
                    rsx! {
                        p { class: "result-count", {locale.t_count("search-results-count", results.len() as i64)} }
                        if results.is_empty() {
                            p { class: "muted", {locale.t("search-no-results")} }
                        }
                        ul { class: "results",
                            for store in results {
                                ResultCard { key: "{store.id}", store, now: now() }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Narrows the list — and the pins (see `MapCtx::name`) — to stores whose
/// name matches what's typed, word by word in any order (`fuzzy::matches`).
#[component]
fn NameSearch() -> Element {
    let locale = use_locale();
    let mut map = use_map();
    let label = locale.t("search-name");

    rsx! {
        label { class: "toolbar-search",
            lucide::Search { size: 16 }
            Input {
                r#type: "search",
                "aria-label": "{label}",
                placeholder: locale.t("search-name-placeholder"),
                value: "{map.name}",
                oninput: move |e: FormEvent| map.name.set(e.value()),
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
    let label = locale.t("search-product");

    rsx! {
        Select::<Option<i64>> {
            class: "toolbar-select",
            "aria-label": "{label}",
            value: Some(value.into()),
            on_value_change: move |v: Option<Option<i64>>| map.product.set(v.flatten()),
            SelectOption::<Option<i64>> {
                index: 0usize,
                value: None,
                text_value: format!("{label}: {}", locale.t("search-all")),
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

/// Farm shop, vending machine or market. Filters the pins too (see
/// `MapCtx::kind`).
#[component]
fn KindFilter() -> Element {
    let locale = use_locale();
    let mut map = use_map();
    let value = use_memo(move || Some((map.kind)()));
    let label = locale.t("store-form-kind");

    rsx! {
        Select::<Option<StoreKind>> {
            class: "toolbar-select",
            "aria-label": "{label}",
            value: Some(value.into()),
            on_value_change: move |v: Option<Option<StoreKind>>| map.kind.set(v.flatten()),
            SelectOption::<Option<StoreKind>> {
                index: 0usize,
                value: None,
                text_value: format!("{label}: {}", locale.t("search-all")),
                {locale.t("search-all")}
            }
            for (i , kind) in StoreKind::ALL.into_iter().enumerate() {
                SelectOption::<Option<StoreKind>> {
                    key: "{kind.as_str()}",
                    index: i + 1,
                    value: Some(kind),
                    text_value: locale.t(kind.label_key()),
                    {locale.t(kind.label_key())}
                }
            }
        }
    }
}

#[component]
fn SortSelect(sort: Signal<Sort>) -> Element {
    let locale = use_locale();
    let value = use_memo(move || Some(sort()));
    let options = [(Sort::Distance, "search-sort-distance"), (Sort::Name, "search-sort-name")];

    rsx! {
        Select::<Sort> {
            class: "toolbar-select toolbar-sort",
            "aria-label": locale.t("search-sort"),
            value: Some(value.into()),
            on_value_change: move |v: Option<Sort>| sort.set(v.unwrap_or(Sort::Distance)),
            for (i , (option , key)) in options.into_iter().enumerate() {
                SelectOption::<Sort> {
                    key: "{key}",
                    index: i,
                    value: option,
                    text_value: locale.t(key),
                    {locale.t(key)}
                }
            }
        }
    }
}

#[component]
fn ResultCard(store: StoreSearchResult, now: Option<(i16, String)>) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let id = store.id;
    let more = (store.product_total - CHIP_LIMIT.min(store.products.len()) as i64).max(0);
    let status = now.and_then(|(day, time)| opening_hours::today_status(&store.openinghours, day, &time));
    let lead_icon = store.products.first().and_then(|p| p.icon.clone()).unwrap_or_else(|| "🌾".into());

    rsx! {
        li {
            class: "result-card",
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
            div { class: "result-thumb",
                match store.photo_id {
                    Some(photo) => rsx! { img { src: "/image/{photo}", alt: "", loading: "lazy" } },
                    None => rsx! { span { "aria-hidden": "true", "{lead_icon}" } },
                }
            }
            div { class: "result-main",
                div { class: "result-head",
                    h3 { class: "result-name", "{store.name}" }
                    if let Some(m) = store.distance_m {
                        span { class: "result-distance", {format!("{:.1} km", m / 1000.0)} }
                    }
                }
                div { class: "result-chips",
                    for p in store.products.iter().take(CHIP_LIMIT) {
                        Badge { class: "result-chip", variant: BadgeVariant::Secondary,
                            "{p.icon.as_deref().unwrap_or(\"📦\")} {p.name}"
                        }
                    }
                    if more > 0 {
                        Badge { class: "result-chip", variant: BadgeVariant::Outline, "+{more}" }
                    }
                }
                div { class: "result-meta",
                    if let Some(place) = &store.place {
                        span { class: "result-place",
                            lucide::MapPin { size: 14 }
                            "{place}"
                        }
                    }
                    match status {
                        Some(TodayStatus::OpenUntil(time)) => rsx! {
                            span { class: "result-status open", {locale.t_name("search-open-until", &time)} }
                        },
                        Some(TodayStatus::OpensAt(time)) => rsx! {
                            span { class: "result-status later", {locale.t_name("search-opens-at", &time)} }
                        },
                        Some(TodayStatus::Closed) => rsx! {
                            span { class: "result-status closed", {locale.t("search-closed-today")} }
                        },
                        None => rsx! {},
                    }
                }
            }
            span { class: "result-chevron", "aria-hidden": "true", lucide::ChevronRight { size: 18 } }
        }
    }
}
