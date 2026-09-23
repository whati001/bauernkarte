//! Editing a catalog product itself — shared by every store offering it.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::toast::{use_toast, ToastOptions};

use crate::{
    api::{
        error::AppError,
        product::{delete_product, product_for_edit, update_product},
    },
    app::Route,
    components::{
        account::RequireLogin,
        common::{use_panel_data, Field, FormError, LoadError, Panel},
        map::use_map,
    },
    i18n::use_locale,
    models::ProductEdit,
    ui::{
        alert_dialog::{AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogTitle},
        button::{Button, ButtonVariant},
        input::Input,
        textarea::Textarea,
    },
};

#[component]
pub fn EditProduct(store_id: i64, id: i64) -> Element {
    rsx! {
        RequireLogin {
            EditProductLoader { store_id, id }
        }
    }
}

#[component]
fn EditProductLoader(store_id: i64, id: i64) -> Element {
    // Reactive and non-suspending in the browser — see `use_panel_data`.
    let product = use_panel_data(use_reactive!(|id| product_for_edit(id)))?;
    match product {
        Some(Ok(product)) => rsx! { EditProductForm { key: "{id}", store_id, product } },
        Some(Err(error)) => rsx! { LoadError { error } },
        None => rsx! {},
    }
}

#[component]
fn EditProductForm(store_id: i64, product: ProductEdit) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let toasts = use_toast();
    let map = use_map();
    let mut name = use_signal(|| product.name.clone());
    let mut description = use_signal(|| product.description.clone().unwrap_or_default());
    let mut error = use_signal(|| None::<AppError>);
    let mut confirm_delete = use_signal(|| false);
    let id = product.id;

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match update_product(id, name(), description()).await {
                Ok(saved) => {
                    toasts.success(locale.t_name("confirmation-updated", &saved), ToastOptions::new());
                    map.reload_results();
                    nav.push(Route::StorePanel { id: store_id });
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        Panel { back: Route::StorePanel { id: store_id }, title: locale.t("edit-product-form-heading"),
            form { class: "form", onsubmit: submit,
                Field { label: locale.t("edit-product-form-name"), html_for: "product-name",
                    Input { id: "product-name", required: true, value: "{name}", oninput: move |e: FormEvent| name.set(e.value()) }
                }
                Field { label: locale.t("edit-product-form-description"), html_for: "product-desc",
                    Textarea { id: "product-desc", value: "{description}", oninput: move |e: FormEvent| description.set(e.value()) }
                }
                FormError { error }
                div { class: "form-actions",
                    Button { r#type: "submit",
                        lucide::CircleCheck { size: 16 }
                        {locale.t("action-save")}
                    }
                    Button { r#type: "button", variant: ButtonVariant::Destructive, onclick: move |_| confirm_delete.set(true),
                        lucide::Trash2 { size: 16 }
                        {locale.t("action-delete")}
                    }
                }
            }
            // Removes the product from the whole catalog, not just from
            // this shop — worth one more click.
            AlertDialog { open: confirm_delete(), on_open_change: move |v| confirm_delete.set(v),
                AlertDialogTitle { {locale.t("edit-product-form-heading")} ": {product.name}" }
                AlertDialogActions {
                    AlertDialogCancel { {locale.t("action-cancel")} }
                    AlertDialogAction {
                        on_click: move |_| async move {
                            if delete_product(id).await.is_ok() {
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
