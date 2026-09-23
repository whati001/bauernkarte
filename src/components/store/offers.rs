//! The products: one row each, marked when it's in season right now;
//! a row opens to show the description, the season across the year, the
//! hearts, and (for signed-in visitors) its edit actions.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};

use super::{use_store, Styles};
use crate::{
    api::offer::{delete_offer, heart_offer, unheart_offer},
    app::{use_session, Route},
    components::map::use_map,
    i18n::use_locale,
    models::OfferDetail,
    seasonality,
    ui::{
        alert_dialog::{AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogDescription, AlertDialogTitle},
        badge::{Badge, BadgeVariant},
        button::{Button, ButtonSize, ButtonVariant},
    },
};

/// The current month, 1–12. In the browser this is the visitor's clock;
/// the server-rendered first paint uses the server's — the same month
/// except for a few hours around midnight at a month's end.
fn this_month() -> i16 {
    time::OffsetDateTime::now_utc().month() as i16
}

/// Only seasonal offers can be "in season" — an all-year product is
/// simply always there, and badging every one of them would drown the
/// badges that mean something.
fn in_season(offer: &OfferDetail, month: i16) -> Option<bool> {
    offer.seasonal_months.as_deref().map(|months| months.contains(&month))
}

#[component]
pub fn OfferList() -> Element {
    let locale = use_locale();
    let store = use_store();
    let session = use_session();
    let d = store.detail.read();
    let month = this_month();
    let in_season_now = d.offers.iter().filter(|o| in_season(o, month) == Some(true)).count();

    rsx! {
        section { class: Styles::section, "aria-labelledby": "products-heading",
            div { class: Styles::section_head,
                h3 { id: "products-heading", class: Styles::section_label, {locale.t("detail-products")} }
                if in_season_now > 0 {
                    span { class: Styles::season_count, {locale.t_count("detail-in-season-count", in_season_now as i64)} }
                }
            }
            if d.offers.is_empty() {
                p { class: Styles::muted, {locale.t("detail-no-products")} }
            }
            ul { class: Styles::offers,
                for offer in d.offers.iter().cloned() {
                    OfferRow { key: "{offer.store_product_id}", offer, month }
                }
            }
            if session.is_logged_in() {
                Link { class: Styles::add_tile.to_string(), to: Route::AddOffer { id: d.id },
                    lucide::CirclePlus { size: 18 }
                    {locale.t("detail-add-product")}
                }
            }
        }
    }
}

#[component]
fn OfferRow(offer: OfferDetail, month: i16) -> Element {
    let locale = use_locale();
    let season = in_season(&offer, month);
    let icon = offer.icon.clone().unwrap_or_else(|| "📦".into());

    rsx! {
        li {
            Collapsible {
                class: if season == Some(false) { format!("{} {}", Styles::offer, Styles::offer_off) } else { Styles::offer.to_string() },
                CollapsibleTrigger { class: Styles::offer_row, "aria-label": "{offer.name} – {locale.t(\"detail-more-actions\")}",
                    span { class: Styles::offer_icon, "aria-hidden": "true", "{icon}" }
                    span { class: Styles::offer_text,
                        span { class: Styles::offer_name, "{offer.name}" }
                        if let Some(desc) = &offer.description {
                            span { class: Styles::offer_desc_short, "{desc}" }
                        }
                    }
                    span { class: Styles::offer_side,
                        match season {
                            Some(true) => rsx! { Badge { class: Styles::season_badge, variant: BadgeVariant::Secondary, {locale.t("detail-in-season")} } },
                            Some(false) => rsx! { Badge { class: Styles::season_badge_off, variant: BadgeVariant::Outline, {locale.t("detail-out-of-season")} } },
                            None => rsx! {},
                        }
                    }
                }
                CollapsibleContent {
                    OfferDetails { offer: offer.clone() }
                }
            }
        }
    }
}

#[component]
fn OfferDetails(offer: OfferDetail) -> Element {
    let locale = use_locale();
    let session = use_session();
    let store = use_store();
    let months = offer.seasonal_months.clone();
    let available = seasonality::availability(months.as_deref());

    rsx! {
        div { class: Styles::offer_details,
            div { class: Styles::season,
                span { class: Styles::season_label,
                    lucide::Leaf { size: 14 }
                    if months.is_none() {
                        {locale.t("detail-year-round")}
                    } else {
                        {seasonality::summary(locale, months.as_deref())}
                    }
                }
                div { class: Styles::season_bar, role: "img", "aria-label": seasonality::summary(locale, months.as_deref()),
                    for (i , on) in available.iter().enumerate() {
                        span {
                            key: "{i}",
                            class: if *on { format!("{} {}", Styles::season_dot, Styles::season_dot_on) } else { Styles::season_dot.to_string() },
                            title: locale.t(seasonality::MONTH_KEYS[i]),
                        }
                    }
                }
            }
            div { class: Styles::offer_actions,
                Hearts { offer: offer.clone() }
                if session.is_logged_in() {
                    Link {
                        class: Styles::icon_link.to_string(),
                        to: Route::EditOffer { id: offer.store_product_id },
                        title: locale.t("detail-edit-seasonality-title"),
                        "aria-label": locale.t("detail-edit-seasonality"),
                        lucide::Leaf { size: 16 }
                    }
                    Link {
                        class: Styles::icon_link.to_string(),
                        to: Route::EditProduct { store_id: store.detail.read().id, id: offer.product_id },
                        title: locale.t("detail-edit-product-title"),
                        "aria-label": locale.t("detail-edit-product"),
                        lucide::Package { size: 16 }
                    }
                    RemoveOffer { offer: offer.clone() }
                }
            }
        }
    }
}

/// The per-offer "UP" heart: a count for everyone, a toggle for
/// signed-in visitors.
#[component]
fn Hearts(offer: OfferDetail) -> Element {
    let locale = use_locale();
    let session = use_session();
    let store = use_store();
    let id = offer.store_product_id;
    let rated = offer.viewer_has_rated_up;

    if !session.is_logged_in() {
        return rsx! {
            span { class: Styles::hearts, title: locale.t("detail-rating-label"),
                lucide::Heart { size: 16 }
                "{offer.hearts}"
            }
        };
    }
    rsx! {
        Button {
            class: if rated { format!("{} {}", Styles::hearts, Styles::hearts_on) } else { Styles::hearts.to_string() },
            variant: ButtonVariant::Outline,
            size: ButtonSize::Sm,
            "aria-pressed": rated,
            title: if rated { locale.t("detail-unrate") } else { locale.t("detail-rate") },
            onclick: move |_| async move {
                let done = if rated { unheart_offer(id).await } else { heart_offer(id).await };
                if done.is_ok() {
                    store.reload();
                }
            },
            lucide::Heart { size: 16 }
            "{offer.hearts}"
        }
    }
}

#[component]
fn RemoveOffer(offer: OfferDetail) -> Element {
    let locale = use_locale();
    let store = use_store();
    let map = use_map();
    let mut open = use_signal(|| false);
    let id = offer.store_product_id;

    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            class: Styles::danger,
            title: locale.t("detail-remove-offer-title"),
            "aria-label": locale.t("detail-remove-offer"),
            onclick: move |_| open.set(true),
            lucide::Trash2 { size: 16 }
        }
        AlertDialog { open: open(), on_open_change: move |v| open.set(v),
            AlertDialogTitle { {locale.t("detail-remove-offer-title")} }
            AlertDialogDescription { {locale.t("detail-remove-offer")} ": {offer.name}" }
            AlertDialogActions {
                AlertDialogCancel { {locale.t("action-cancel")} }
                AlertDialogAction {
                    on_click: move |_| async move {
                        if delete_offer(id).await.is_ok() {
                            store.reload();
                            map.reload_results();
                        }
                    },
                    {locale.t("action-delete")}
                }
            }
        }
    }
}
