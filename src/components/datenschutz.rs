//! Application-specific privacy disclosure backed by the independent
//! `privacy_info` singleton.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use crate::{
    api::privacy::privacy_info,
    app::Route,
    components::common::{LoadError, Panel, Section},
    i18n::use_locale,
};

#[component]
pub fn Datenschutz() -> dioxus::prelude::Element {
    let locale = use_locale();
    let info = use_server_future(privacy_info)?;
    let info = match info() {
        Some(Ok(info)) => info,
        Some(Err(error)) => return rsx! { LoadError { error } },
        None => return rsx! {},
    };

    rsx! {
        document::Title { {locale.t("datenschutz-heading")} }
        Panel {
            back: Route::SearchPanel {},
            back_label: locale.t("action-back-to-search"),
            title: locale.t("datenschutz-heading"),
            p { {locale.t("datenschutz-intro")} }
            p { class: "muted", {locale.t("datenschutz-updated")} }

            Section { title: locale.t("datenschutz-controller-heading"),
                p { {locale.t("datenschutz-controller-body")} }
                if !info.is_configured() {
                    div { class: "form-error", role: "alert",
                        lucide::TriangleAlert { size: 16 }
                        {locale.t("datenschutz-controller-unconfigured")}
                    }
                } else {
                    address { class: "impressum-address",
                        strong { "{info.controller_name}" }
                        if !info.street.is_empty() { span { "{info.street}" } }
                        if !info.postal_city().is_empty() { span { {info.postal_city()} } }
                        if !info.country.is_empty() { span { "{info.country}" } }
                    }
                    if !info.email.is_empty() || !info.phone.is_empty() {
                        dl { class: "facts-list",
                            if !info.email.is_empty() {
                                dt { {locale.t("datenschutz-controller-email")} }
                                dd { a { href: "mailto:{info.email}", "{info.email}" } }
                            }
                            if !info.phone.is_empty() {
                                dt { {locale.t("datenschutz-controller-phone")} }
                                dd { a { href: "tel:{info.phone}", "{info.phone}" } }
                            }
                        }
                    }
                }
            }

            Section { title: locale.t("datenschutz-technical-heading"),
                p { {locale.t("datenschutz-technical-body")} }
            }
            Section { title: locale.t("datenschutz-account-heading"),
                p { {locale.t("datenschutz-account-body")} }
            }
            Section { title: locale.t("datenschutz-content-heading"),
                p { {locale.t("datenschutz-content-body")} }
                p { strong { {locale.t("datenschutz-public-warning")} } }
            }
            Section { title: locale.t("datenschutz-location-heading"),
                p { {locale.t("datenschutz-location-body")} }
            }
            Section { title: locale.t("datenschutz-osm-heading"),
                p { {locale.t("datenschutz-osm-body")} }
                p {
                    a {
                        href: "https://osmfoundation.org/wiki/Privacy_Policy",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        {locale.t("datenschutz-osm-policy")}
                    }
                }
                p { {locale.t("datenschutz-google-body")} }
            }
            Section { title: locale.t("datenschutz-storage-heading"),
                ul {
                    li { {locale.t("datenschutz-storage-session")} }
                    li { {locale.t("datenschutz-storage-locale")} }
                    li { {locale.t("datenschutz-storage-sidebar")} }
                    li { {locale.t("datenschutz-storage-pwa")} }
                    li { {locale.t("datenschutz-storage-osm-cache")} }
                }
                p { {locale.t("datenschutz-storage-legal")} }
            }
            Section { title: locale.t("datenschutz-recipients-heading"),
                p { {locale.t("datenschutz-recipients-body")} }
            }
            Section { title: locale.t("datenschutz-retention-heading"),
                p { {locale.t("datenschutz-retention-body")} }
            }
            Section { title: locale.t("datenschutz-rights-heading"),
                ul {
                    li { {locale.t("datenschutz-right-information")} }
                    li { {locale.t("datenschutz-right-access")} }
                    li { {locale.t("datenschutz-right-rectification")} }
                    li { {locale.t("datenschutz-right-erasure")} }
                    li { {locale.t("datenschutz-right-restriction")} }
                    li { {locale.t("datenschutz-right-portability")} }
                    li { {locale.t("datenschutz-right-objection")} }
                    li { {locale.t("datenschutz-right-withdrawal")} }
                }
                p { {locale.t("datenschutz-rights-contact")} }
                p { {locale.t("datenschutz-no-automated-decisions")} }
            }
            Section { title: locale.t("datenschutz-complaint-heading"),
                p { {locale.t("datenschutz-complaint")} }
                address { class: "impressum-address",
                    strong { {locale.t("datenschutz-dsb-name")} }
                    span { {locale.t("datenschutz-dsb-address")} }
                    a { href: "mailto:dsb@dsb.gv.at", "dsb@dsb.gv.at" }
                    a {
                        href: "https://dsb.gv.at/ueber-die-datenschutzbehoerde/kontakt",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        {locale.t("datenschutz-dsb-link")}
                    }
                }
            }
        }
    }
}
