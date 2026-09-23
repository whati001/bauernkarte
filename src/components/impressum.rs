//! The Impressum, from the single `site_info` row (edited under
//! `/admin/site-info`). Blank fields are left out, not shown empty.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use crate::{
    api::site::impressum,
    app::Route,
    components::common::{LoadError, Panel, Section},
    i18n::use_locale,
};

#[component]
pub fn Impressum() -> Element {
    let locale = use_locale();
    let info = use_server_future(impressum)?;
    let info = match info() {
        Some(Ok(info)) => info,
        Some(Err(error)) => return rsx! { LoadError { error } },
        None => return rsx! {},
    };

    rsx! {
        document::Title { {locale.t("impressum-heading")} }
        Panel { back: Route::SearchPanel {}, back_label: locale.t("action-back-to-search"), title: locale.t("impressum-heading"),
            if !info.is_configured() {
                div { class: "form-error", role: "alert",
                    lucide::TriangleAlert { size: 16 }
                    {locale.t("impressum-unconfigured")}
                }
            } else {
                Section { title: locale.t("impressum-operator"),
                    address { class: "impressum-address",
                        strong { "{info.operator_name}" }
                        if !info.street.is_empty() { span { "{info.street}" } }
                        if !info.postal_city().is_empty() { span { {info.postal_city()} } }
                        if !info.country.is_empty() { span { "{info.country}" } }
                    }
                }
                if !info.email.is_empty() || !info.phone.is_empty() {
                    Section { title: locale.t("impressum-contact"),
                        dl { class: "facts-list",
                            if !info.email.is_empty() {
                                dt { {locale.t("auth-email")} }
                                dd { a { href: "mailto:{info.email}", "{info.email}" } }
                            }
                            if !info.phone.is_empty() {
                                dt { {locale.t("impressum-phone")} }
                                dd { a { href: "tel:{info.phone}", "{info.phone}" } }
                            }
                        }
                    }
                }
                if !info.vat_id.is_empty() || !info.register_number.is_empty() || !info.responsible.is_empty() {
                    Section { title: locale.t("impressum-legal"),
                        dl { class: "facts-list",
                            if !info.vat_id.is_empty() {
                                dt { {locale.t("impressum-vat-id")} }
                                dd { "{info.vat_id}" }
                            }
                            if !info.register_number.is_empty() {
                                dt { {locale.t("impressum-register")} }
                                dd { "{info.register_number}" }
                            }
                            if !info.responsible.is_empty() {
                                dt { {locale.t("impressum-responsible")} }
                                dd { "{info.responsible}" }
                            }
                        }
                    }
                }
                if !info.purpose.is_empty() {
                    Section { title: locale.t("impressum-purpose"),
                        p { "{info.purpose}" }
                    }
                }
                Section { title: locale.t("impressum-data-heading"),
                    p { {locale.t("impressum-data-osm")} }
                }
            }
        }
    }
}
