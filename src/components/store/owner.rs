//! Who runs the shop: portrait (or initials), since when, a few words,
//! and the phone number.

use dioxus::prelude::*;

use super::{use_store, Styles};
use crate::{
    i18n::use_locale,
    ui::avatar::{Avatar, AvatarFallback, AvatarImage, AvatarImageSize},
};

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn OwnerSection() -> Element {
    let locale = use_locale();
    let store = use_store();
    let d = store.detail.read();
    let Some(name) = d.owner_name.clone() else {
        return rsx! {};
    };

    rsx! {
        section { class: Styles::section, "aria-labelledby": "owner-heading",
            h3 { id: "owner-heading", class: Styles::section_label, {locale.t("detail-shop-owner")} }
            div { class: Styles::owner,
                Avatar { class: Styles::owner_avatar, size: AvatarImageSize::Large,
                    if let Some(image) = d.owner_image_id {
                        AvatarImage { src: "/image/{image}", alt: "{name}" }
                    }
                    AvatarFallback { {initials(&name)} }
                }
                div { class: Styles::owner_text,
                    p { class: Styles::owner_name, "{name}" }
                    if let Some(year) = d.owner_since {
                        p { class: Styles::owner_since, {locale.t_year("detail-owner-since", year)} }
                    }
                    if let Some(bio) = &d.owner_bio {
                        p { class: Styles::owner_bio, "{bio}" }
                    }
                    if let Some(phone) = &d.phone {
                        a {
                            class: Styles::owner_phone,
                            href: "tel:{phone}",
                            "aria-label": locale.t_name("detail-call", &name),
                            span { "aria-hidden": "true", "📞" }
                            "{phone}"
                        }
                    }
                }
            }
        }
    }
}
