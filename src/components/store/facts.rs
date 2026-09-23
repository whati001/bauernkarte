//! What the shop sells at a glance, its star rating, where it is and when
//! it's open.

use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};

use super::{directions_url, use_store, Styles};
use crate::{
    api::store::{remove_review, review_store},
    app::use_session,
    i18n::use_locale,
    opening_hours,
    ui::{
        badge::{Badge, BadgeVariant},
        toggle_group::{ToggleGroup, ToggleItem},
    },
};

/// How many product chips before "+N".
const CHIP_LIMIT: usize = 3;

#[component]
pub fn StoreFacts() -> Element {
    let locale = use_locale();
    let store = use_store();
    let session = use_session();
    let d = store.detail.read();
    let more = d.offers.len().saturating_sub(CHIP_LIMIT);

    rsx! {
        div { class: Styles::facts,
            if !d.offers.is_empty() {
                div { class: Styles::chips,
                    for offer in d.offers.iter().take(CHIP_LIMIT) {
                        Badge { key: "{offer.store_product_id}", class: Styles::chip, variant: BadgeVariant::Secondary,
                            "{offer.name}"
                        }
                    }
                    if more > 0 {
                        Badge { class: Styles::chip, variant: BadgeVariant::Outline, "+{more}" }
                    }
                }
            }
            RatingSummary {}
            if session.is_logged_in() {
                ReviewPicker {}
            }
            a {
                class: Styles::fact,
                href: directions_url(d.lat, d.lon),
                target: "_blank",
                rel: "noopener",
                title: locale.t("detail-get-directions"),
                span { class: Styles::fact_icon, "aria-hidden": "true", "📍" }
                match &d.address {
                    Some(address) => rsx! { span { "{address}" } },
                    None => rsx! { span { {format!("{:.5}, {:.5}", d.lat, d.lon)} } },
                }
            }
            if !d.openinghours.is_empty() {
                OpeningHours {}
            }
            // With an owner, the number sits in their block instead.
            if let (None, Some(phone)) = (&d.owner_name, &d.phone) {
                a { class: Styles::fact, href: "tel:{phone}",
                    span { class: Styles::fact_icon, "aria-hidden": "true", "📞" }
                    span { "{phone}" }
                }
            }
        }
    }
}

/// ★★★★★ 4.7 (138 reviews). Stars are drawn twice — an outline row and a
/// filled row clipped to the average — so a 4.7 shows as 4.7 stars.
#[component]
fn RatingSummary() -> Element {
    let locale = use_locale();
    let store = use_store();
    let review = store.detail.read().review.clone();
    let Some(average) = review.average else {
        return rsx! {
            p { class: Styles::rating_empty, {locale.t("detail-no-reviews")} }
        };
    };
    let percent = (average / 5.0 * 100.0).clamp(0.0, 100.0);
    let average_text = match locale {
        crate::i18n::Locale::De => format!("{average:.1}").replace('.', ","),
        crate::i18n::Locale::En => format!("{average:.1}"),
    };

    rsx! {
        div { class: Styles::rating,
            span {
                class: Styles::stars,
                role: "img",
                "aria-label": "{average_text} / 5",
                span { class: Styles::stars_empty,
                    for _ in 0..5 { lucide::Star { size: 16 } }
                }
                span { class: Styles::stars_full, style: "width: {percent}%",
                    for _ in 0..5 { lucide::Star { size: 16 } }
                }
            }
            strong { class: Styles::rating_value, "{average_text}" }
            span { class: Styles::rating_count, {locale.t_count("detail-reviews-count", review.count)} }
        }
    }
}

/// The visitor's own 1–5 stars. Choosing again replaces it; choosing the
/// current value again removes it.
#[component]
fn ReviewPicker() -> Element {
    let locale = use_locale();
    let mut store = use_store();
    let mine = store.detail.read().review.mine.unwrap_or(0) as usize;
    let id = store.detail.read().id;

    let set_stars = move |pressed: HashSet<usize>| {
        spawn(async move {
            let result = match pressed.iter().next() {
                Some(&stars) => review_store(id, stars as i16).await,
                None => remove_review(id).await,
            };
            if let Ok(review) = result {
                store.detail.write().review = review;
            }
        });
    };

    rsx! {
        div { class: Styles::review_picker,
            span { class: Styles::review_label, {locale.t("detail-your-rating")} }
            ToggleGroup {
                horizontal: true,
                pressed: Some(if mine > 0 { HashSet::from([mine]) } else { HashSet::new() }),
                on_pressed_change: set_stars,
                for n in 1..=5usize {
                    ToggleItem {
                        key: "{n}",
                        index: n,
                        class: if n <= mine { format!("{} {}", Styles::review_star, Styles::review_star_on) } else { Styles::review_star.to_string() },
                        title: locale.t_count("detail-stars", n as i64),
                        "aria-label": locale.t_count("detail-stars", n as i64),
                        lucide::Star { size: 18 }
                    }
                }
            }
        }
    }
}

/// "Fri–Sun 9:00–17:00" up front; the whole week one tap away.
#[component]
fn OpeningHours() -> Element {
    let locale = use_locale();
    let store = use_store();
    let hours = store.detail.read().openinghours.clone();
    let lines = opening_hours::summary_lines(locale, &hours);
    let week = opening_hours::week_rows(locale, &hours);

    rsx! {
        Collapsible { class: Styles::hours,
            div { class: Styles::fact,
                span { class: Styles::fact_icon, "aria-hidden": "true", "🕘" }
                span { class: Styles::hours_lines,
                    for line in lines {
                        span { "{line}" }
                    }
                }
                CollapsibleTrigger { class: Styles::hours_toggle,
                    {locale.t("detail-hours-show-week")}
                    lucide::ChevronDown { size: 14 }
                }
            }
            CollapsibleContent {
                table { class: Styles::week,
                    tbody {
                        for row in week {
                            tr { key: "{row.day}",
                                th { scope: "row", "{row.label}" }
                                td {
                                    match row.range {
                                        Some(range) => rsx! { "{range}" },
                                        None => rsx! { span { class: Styles::closed, {locale.t("opening-hours-closed")} } },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
