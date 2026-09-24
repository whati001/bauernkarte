//! Uploading a photo of a shop, or a portrait of whoever runs it.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::{
    checkbox::CheckboxState,
    toast::{use_toast, ToastOptions},
};

use crate::{
    api::{error::AppError, image::upload_image},
    app::Route,
    components::{
        account::RequireLogin,
        common::{Field, FormError, Panel},
    },
    i18n::use_locale,
    ui::{button::Button, checkbox::Checkbox, input::Input, label::Label},
};

#[component]
pub fn AddPhoto(id: i64) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let toasts = use_toast();
    let mut error = use_signal(|| None::<AppError>);
    let mut uploading = use_signal(|| false);
    // A portrait is never the store image, so ticking one clears the other.
    let mut is_owner = use_signal(|| false);
    let mut is_cover = use_signal(|| false);
    let state = |on: bool| Some(if on { CheckboxState::Checked } else { CheckboxState::Unchecked });

    // Sent as-is as multipart: the file never passes through Rust on the
    // client side.
    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            uploading.set(true);
            let result = upload_image(id, evt.into()).await;
            uploading.set(false);
            match result {
                Ok(()) => {
                    // Pending approval, so the panel wouldn't change — say
                    // so rather than leave the uploader wondering.
                    toasts.success(locale.t("confirmation-image-pending"), ToastOptions::new());
                    nav.push(Route::StorePanel { id });
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        RequireLogin {
            Panel { back: Route::StorePanel { id }, title: locale.t("image-form-heading"),
                form { class: "form", enctype: "multipart/form-data", onsubmit: submit,
                    Field { label: locale.t("image-form-file-label"), html_for: "image-file",
                        Input {
                            id: "image-file",
                            r#type: "file",
                            name: "file",
                            accept: "image/jpeg,image/png,image/webp",
                            required: true,
                        }
                    }
                    Field { label: locale.t("image-form-description-optional"), html_for: "image-description",
                        Input { id: "image-description", name: "description" }
                    }
                    div { class: "switch-field",
                        Checkbox {
                            id: "image-cover",
                            name: "is_cover",
                            checked: state(is_cover()),
                            disabled: is_owner(),
                            on_checked_change: move |s| is_cover.set(s == CheckboxState::Checked),
                        }
                        Label { html_for: "image-cover", {locale.t("image-form-is-cover")} }
                    }
                    p { class: "field-hint", {locale.t("image-form-is-cover-hint")} }
                    div { class: "switch-field",
                        Checkbox {
                            id: "image-owner",
                            name: "is_owner",
                            checked: state(is_owner()),
                            on_checked_change: move |s| {
                                let on = s == CheckboxState::Checked;
                                is_owner.set(on);
                                if on {
                                    is_cover.set(false);
                                }
                            },
                        }
                        Label { html_for: "image-owner", {locale.t("image-form-is-owner")} }
                    }
                    FormError { error }
                    Button { r#type: "submit", disabled: uploading(),
                        lucide::Upload { size: 16 }
                        {locale.t("image-form-upload")}
                    }
                }
            }
        }
    }
}
