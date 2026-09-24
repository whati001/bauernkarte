//! The top bar: brand, the product picker, the account menu and the
//! language switch, plus the row of most-popular product chips.

use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_icons::lucide;

use crate::{
    api::{search::top_products, session::logout},
    app::{use_session, Route},
    components::{map::MapCtx, shell::use_catalog},
    fuzzy,
    i18n::{use_locale, Locale},
    models::CatalogProduct,
    ui::{
        button::{Button, ButtonSize, ButtonVariant},
        combobox::{Combobox, ComboboxEmpty, ComboboxOption},
        dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger},
        toggle_group::{ToggleGroup, ToggleItem},
    },
};

/// `with_search` is off in the admin area, which has no map to filter.
#[component]
pub fn Navbar(with_search: bool) -> Element {
    let locale = use_locale();
    // Phones only: the picker hides behind a magnifier and, once opened,
    // takes over the whole top row. Wider screens always show it.
    let mut search_open = use_signal(|| false);
    use_effect(move || {
        if search_open() {
            document::eval("document.querySelector('.nav-search input')?.focus();");
        }
    });
    rsx! {
        nav { id: "navbar",
            div { class: if search_open() { "nav-main search-open" } else { "nav-main" },
                Link { class: "brand", to: Route::SearchPanel {},
                    span { class: "brand-mark", lucide::Sprout { size: 20 } }
                    span { class: "brand-text",
                        span { class: "brand-name", {locale.t("nav-brand")} }
                        span { class: "brand-tagline", {locale.t("nav-tagline")} }
                    }
                }
                if with_search {
                    button {
                        class: "nav-icon-link nav-search-close",
                        r#type: "button",
                        title: locale.t("nav-search-close"),
                        "aria-label": locale.t("nav-search-close"),
                        onclick: move |_| search_open.set(false),
                        lucide::ArrowLeft { size: 18 }
                    }
                    ProductPicker { on_pick: move |_| search_open.set(false) }
                    button {
                        class: "nav-icon-link nav-search-toggle",
                        r#type: "button",
                        title: locale.t("nav-search-label"),
                        "aria-label": locale.t("nav-search-label"),
                        "aria-expanded": "{search_open}",
                        onclick: move |_| search_open.set(true),
                        lucide::Search { size: 18 }
                    }
                }
                AccountActions {}
            }
            if with_search {
                PopularProducts {}
            }
        }
    }
}

/// A product *picker*, not a free-text search: typing only narrows the
/// list, and the filter changes when an entry is chosen. Choosing lands
/// on the search panel from wherever the visitor was.
#[component]
fn ProductPicker(on_pick: EventHandler) -> Element {
    let locale = use_locale();
    let mut map = use_context::<MapCtx>();
    let catalog = use_catalog();
    let nav = navigator();
    let route = use_route::<Route>();
    let selected = use_memo(move || (map.product)());

    rsx! {
        div { class: "nav-search",
            span { class: "nav-search-icon", "aria-hidden": "true",
                match selected().and_then(|id| catalog().into_iter().find(|p| p.id == id)) {
                    // The picked product's emoji stands in for the magnifier,
                    // so the box shows the same icon + name as its chip.
                    Some(product) => rsx! { "{product.icon_or_default()}" },
                    None => rsx! { lucide::Search { size: 18 } },
                }
            }
            Combobox::<i64> {
                value: Some(selected.into()),
                on_value_change: move |id: Option<i64>| {
                    map.product.set(id);
                    on_pick.call(());
                    if route != (Route::SearchPanel {}) {
                        nav.push(Route::SearchPanel {});
                    }
                },
                filter: Callback::new(|(query, text): (String, String)| fuzzy::matches(&query, &text)),
                placeholder: locale.t("nav-search-placeholder"),
                aria_label: locale.t("nav-search-label"),
                list_aria_label: locale.t("nav-search-label"),
                for (index , product) in catalog().into_iter().enumerate() {
                    ComboboxOption::<i64> {
                        key: "{product.id}",
                        index,
                        value: product.id,
                        text_value: product.name.clone(),
                        "{product.icon_or_default()} {product.name}"
                    }
                }
                ComboboxEmpty { {locale.t("nav-search-no-matches")} }
            }
            if selected().is_some() {
                Button {
                    class: "nav-search-clear",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    title: locale.t("nav-search-clear"),
                    "aria-label": locale.t("nav-search-clear"),
                    onclick: move |_| map.product.set(None),
                    lucide::X { size: 16 }
                }
            }
        }
    }
}

/// The quick-pick row: the most-rated products, one tap each. Index 0 is
/// "all products"; tapping the active chip again clears the filter.
#[component]
fn PopularProducts() -> Element {
    let locale = use_locale();
    let mut map = use_context::<MapCtx>();
    let products = use_resource(top_products);
    let products: Vec<CatalogProduct> = products().and_then(Result::ok).unwrap_or_default();
    if products.is_empty() {
        return rsx! {};
    }

    let pressed_index = match (map.product)() {
        None => 0,
        Some(id) => products.iter().position(|p| p.id == id).map_or(usize::MAX, |i| i + 1),
    };
    let ids: Vec<i64> = products.iter().map(|p| p.id).collect();

    rsx! {
        div { class: "nav-products", role: "group", "aria-label": locale.t("nav-products-label"),
            ToggleGroup {
                horizontal: true,
                pressed: Some(HashSet::from([pressed_index])),
                on_pressed_change: move |pressed: HashSet<usize>| {
                    let product = pressed.iter().next().and_then(|&i| i.checked_sub(1)).and_then(|i| ids.get(i).copied());
                    map.product.set(product);
                },
                ToggleItem { index: 0usize, class: "product-pick",
                    lucide::Layers { size: 16 }
                    span { {locale.t("nav-products-all")} }
                }
                for (i , p) in products.iter().enumerate() {
                    ToggleItem { key: "{p.id}", index: i + 1, class: "product-pick",
                        span { "aria-hidden": "true", "{p.icon_or_default()}" }
                        span { "{p.name}" }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum MenuAction {
    Account,
    NewStore,
    Logout,
}

#[component]
fn AccountActions() -> Element {
    let locale = use_locale();
    let mut session = use_session();
    let nav = navigator();

    let Some(user) = session.user() else {
        return rsx! {
            div { class: "nav-actions",
                Link { class: "nav-cta", to: Route::Login {}, title: locale.t("nav-login"),
                    lucide::LogIn { size: 18 }
                    span { class: "sr-only", {locale.t("nav-login")} }
                }
                LangSwitch {}
            }
        };
    };

    let on_select = move |action: MenuAction| match action {
        MenuAction::Account => {
            nav.push(Route::Account {});
        }
        MenuAction::NewStore => {
            nav.push(Route::NewStore {});
        }
        MenuAction::Logout => {
            spawn(async move {
                if logout().await.is_ok() {
                    session.0.set(None);
                    nav.push(Route::SearchPanel {});
                }
            });
        }
    };

    rsx! {
        div { class: "nav-actions",
            if user.admin {
                Link {
                    class: "nav-icon-link",
                    to: Route::AdminIndex {},
                    title: locale.t("admin-title"),
                    "aria-label": locale.t("admin-title"),
                    lucide::HardHat { size: 18 }
                }
            }
            DropdownMenu {
                DropdownMenuTrigger { class: "nav-user",
                    title: locale.t_name("nav-account-of", &user.name),
                    lucide::User { size: 16 }
                    span { class: "nav-user-name", "{user.name}" }
                    lucide::ChevronDown { size: 14 }
                }
                DropdownMenuContent {
                    DropdownMenuItem::<MenuAction> { index: 0usize, value: MenuAction::Account, on_select,
                        lucide::User { size: 16 }
                        {locale.t("nav-account")}
                    }
                    DropdownMenuItem::<MenuAction> { index: 1usize, value: MenuAction::NewStore, on_select,
                        lucide::Store { size: 16 }
                        {locale.t("nav-new-store")}
                    }
                    DropdownMenuItem::<MenuAction> { index: 2usize, value: MenuAction::Logout, on_select,
                        lucide::LogOut { size: 16 }
                        {locale.t("nav-logout")}
                    }
                }
            }
            LangSwitch {}
        }
    }
}

/// A real navigation, not a router link: switching language reloads the
/// page on purpose (everything on screen needs retranslating), via the
/// server route that sets the cookie.
#[component]
fn LangSwitch() -> Element {
    let current = use_locale();
    rsx! {
        span { class: "lang-switch", role: "group", "aria-label": "Sprache / Language",
            for locale in [Locale::De, Locale::En] {
                a {
                    href: "/locale/{locale.code()}",
                    class: if locale == current { "lang-link active" } else { "lang-link" },
                    "aria-current": if locale == current { "true" } else { "false" },
                    {locale.code().to_uppercase()}
                }
            }
        }
    }
}
