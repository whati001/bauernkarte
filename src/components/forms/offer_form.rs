//! Adding a product to a store, and editing when one of its products is
//! in season.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::toast::{use_toast, ToastOptions};

use super::{OfferState, ProductChoiceFields, SeasonFields, SeasonState};
use crate::{
    api::{
        error::AppError,
        offer::{add_offer, offer_for_edit, update_offer},
    },
    app::Route,
    components::{
        account::RequireLogin,
        common::{use_panel_data, FormError, LoadError, Panel},
        map::use_map,
    },
    i18n::use_locale,
    models::OfferEdit,
    ui::button::Button,
};

#[component]
pub fn AddOffer(id: i64) -> Element {
    rsx! {
        RequireLogin {
            AddOfferForm { key: "{id}", store_id: id }
        }
    }
}

#[component]
fn AddOfferForm(store_id: i64) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let toasts = use_toast();
    let map = use_map();
    let offer = use_hook(OfferState::new);
    let mut error = use_signal(|| None::<AppError>);
    let mut saving = use_signal(|| false);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            let input = match offer.to_input() {
                Ok(input) => input,
                Err(key) => return error.set(Some(AppError::invalid(key))),
            };
            saving.set(true);
            let result = add_offer(store_id, input).await;
            saving.set(false);
            match result {
                Ok(product) => {
                    toasts.success(locale.t_name("confirmation-pending", &product), ToastOptions::new());
                    map.reload_results();
                    nav.push(Route::StorePanel { id: store_id });
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        Panel { back: Route::StorePanel { id: store_id }, title: locale.t("detail-add-product"),
            form { class: "form", onsubmit: submit,
                ProductChoiceFields { id: "offer", state: offer }
                SeasonFields { id: "offer", state: offer.season }
                FormError { error }
                Button { r#type: "submit", disabled: saving(),
                    lucide::CircleCheck { size: 16 }
                    {locale.t("action-save")}
                }
            }
        }
    }
}

#[component]
pub fn EditOffer(id: i64) -> Element {
    rsx! {
        RequireLogin {
            EditOfferLoader { id }
        }
    }
}

#[component]
fn EditOfferLoader(id: i64) -> Element {
    // Reactive and non-suspending in the browser — see `use_panel_data`.
    let offer = use_panel_data(use_reactive!(|id| offer_for_edit(id)))?;
    match offer {
        Some(Ok(offer)) => rsx! { EditOfferForm { key: "{id}", offer } },
        Some(Err(error)) => rsx! { LoadError { error } },
        None => rsx! {},
    }
}

#[component]
fn EditOfferForm(offer: OfferEdit) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let initial = offer.clone();
    let season = use_hook(move || SeasonState::new(initial.seasonal_months.as_deref()));
    let mut error = use_signal(|| None::<AppError>);
    let mut saving = use_signal(|| false);
    let id = offer.store_product_id;

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            let months = match season.value() {
                Ok(months) => months,
                Err(key) => return error.set(Some(AppError::invalid(key))),
            };
            saving.set(true);
            let result = update_offer(id, months).await;
            saving.set(false);
            match result {
                Ok(store_id) => {
                    nav.push(Route::StorePanel { id: store_id });
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        Panel {
            back: Route::StorePanel { id: offer.store_id },
            title: locale.t_name("store-product-seasonality-form-heading", &offer.product_name),
            form { class: "form", onsubmit: submit,
                SeasonFields { id: "offer", state: season }
                FormError { error }
                Button { r#type: "submit", disabled: saving(),
                    lucide::CircleCheck { size: 16 }
                    {locale.t("action-save")}
                }
            }
        }
    }
}
