//! The panel's closing parts: the photo strip, and the actions — route
//! planning for everyone, editing for signed-in visitors.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use super::{directions_url, use_store, Styles};
use crate::{
    api::store::delete_store,
    app::{use_session, Route},
    components::map::use_map,
    i18n::use_locale,
    ui::{
        alert_dialog::{AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogDescription, AlertDialogTitle},
        button::{Button, ButtonVariant},
    },
};

#[component]
pub fn PhotoStrip() -> Element {
    let locale = use_locale();
    let store = use_store();
    let d = store.detail.read();
    rsx! {
        section { class: Styles::section, "aria-labelledby": "photos-heading",
            h3 { id: "photos-heading", class: Styles::section_label, {locale.t("detail-photos")} }
            div { class: Styles::photos,
                for photo in d.photos.iter() {
                    a { key: "{photo.id}", class: Styles::photo, href: "/image/{photo.id}", target: "_blank", rel: "noopener",
                        img {
                            src: "/image/{photo.id}",
                            alt: photo.description.clone().unwrap_or_else(|| d.name.clone()),
                            loading: "lazy",
                        }
                        if let Some(caption) = &photo.description {
                            span { class: Styles::photo_caption, "{caption}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn StoreActions() -> Element {
    let locale = use_locale();
    let store = use_store();
    let session = use_session();
    let map = use_map();
    let nav = navigator();
    let mut confirm_delete = use_signal(|| false);
    let (id, name, lat, lon) = {
        let d = store.detail.read();
        (d.id, d.name.clone(), d.lat, d.lon)
    };

    rsx! {
        div { class: Styles::actions,
            a { class: Styles::directions, href: directions_url(lat, lon), target: "_blank", rel: "noopener",
                lucide::Navigation { size: 16 }
                {locale.t("detail-get-directions")}
            }
            if session.is_logged_in() {
                div { class: Styles::manage,
                    Link { class: Styles::manage_link.to_string(), to: Route::EditStore { id },
                        lucide::Pencil { size: 16 }
                        {locale.t("action-edit")}
                    }
                    Link { class: Styles::manage_link.to_string(), to: Route::AddPhoto { id },
                        lucide::ImagePlus { size: 16 }
                        {locale.t("detail-add-image")}
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        class: format!("{} {}", Styles::manage_link, Styles::danger),
                        onclick: move |_| confirm_delete.set(true),
                        lucide::Trash2 { size: 16 }
                        {locale.t("action-delete")}
                    }
                }
                AlertDialog { open: confirm_delete(), on_open_change: move |v| confirm_delete.set(v),
                    AlertDialogTitle { {locale.t("detail-delete-store")} }
                    AlertDialogDescription { "{name}" }
                    AlertDialogActions {
                        AlertDialogCancel { {locale.t("action-cancel")} }
                        AlertDialogAction {
                            on_click: move |_| async move {
                                if delete_store(id).await.is_ok() {
                                    map.reload_results();
                                    nav.push(Route::SearchPanel {});
                                }
                            },
                            {locale.t("action-delete")}
                        }
                    }
                }
            }
        }
    }
}
