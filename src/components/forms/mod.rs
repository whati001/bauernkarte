//! Everything a signed-in visitor can submit or edit: stores, offers,
//! products and photos. The field groups several forms share live here.

mod offer_form;
mod photo_form;
mod product_form;
mod store_form;

pub use offer_form::{AddOffer, EditOffer};
pub use photo_form::AddPhoto;
pub use product_form::EditProduct;
pub use store_form::{EditStore, NewStore};

use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{common::Field, shell::use_catalog},
    i18n::use_locale,
    models::{OfferInput, ProductChoice},
    seasonality,
    ui::{
        input::Input,
        label::Label,
        select::{Select, SelectOption},
        switch::Switch,
        textarea::Textarea,
        toggle_group::{ToggleGroup, ToggleItem},
    },
};

/// A switch with its label beside it.
#[component]
pub fn SwitchField(#[props(into)] id: String, #[props(into)] label: String, checked: Signal<bool>) -> Element {
    let mut checked = checked;
    rsx! {
        div { class: "switch-field",
            Switch {
                id: "{id}",
                checked: Some(checked()),
                on_checked_change: move |v| checked.set(v),
            }
            Label { html_for: "{id}", {label} }
        }
    }
}

/// "Only available seasonally" and, when on, the twelve months to pick
/// from. Off means all year, whatever the grid says.
#[derive(Clone, Copy, PartialEq)]
pub struct SeasonState {
    pub seasonal: Signal<bool>,
    pub months: Signal<[bool; 12]>,
}

impl SeasonState {
    pub fn new(seasonal_months: Option<&[i16]>) -> Self {
        Self {
            seasonal: Signal::new(seasonal_months.is_some()),
            // All ticked by default, so turning "seasonal" on means
            // unticking the off months rather than building a list.
            months: Signal::new(seasonality::availability(seasonal_months)),
        }
    }

    pub fn value(&self) -> Result<Option<Vec<i16>>, &'static str> {
        seasonality::from_form((self.seasonal)(), &(self.months)())
    }
}

#[component]
pub fn SeasonFields(#[props(into)] id: String, state: SeasonState) -> Element {
    let locale = use_locale();
    let mut months = state.months;
    let pressed: HashSet<usize> = (0..12).filter(|&i| months()[i]).collect();
    rsx! {
        SwitchField { id: "{id}-seasonal", label: locale.t("product-form-seasonal-checkbox"), checked: state.seasonal }
        if (state.seasonal)() {
            ToggleGroup {
                class: "month-grid",
                horizontal: true,
                allow_multiple_pressed: true,
                pressed: Some(pressed),
                on_pressed_change: move |set: HashSet<usize>| months.set(std::array::from_fn(|i| set.contains(&i))),
                for (i , key) in seasonality::MONTH_KEYS.iter().enumerate() {
                    ToggleItem { key: "{i}", index: i, class: "month-chip", {locale.t(key)} }
                }
            }
        } else {
            p { class: "field-hint", {locale.t("product-form-seasonal-hint")} }
        }
    }
}

/// Which product an offer is for: pick one from the catalog, or name a
/// new one (submitted for approval along with the offer).
#[derive(Clone, Copy, PartialEq)]
pub struct OfferState {
    pub is_new: Signal<bool>,
    pub existing: Signal<Option<i64>>,
    pub new_name: Signal<String>,
    pub new_description: Signal<String>,
    pub season: SeasonState,
}

impl OfferState {
    pub fn new() -> Self {
        Self {
            is_new: Signal::new(false),
            existing: Signal::new(None),
            new_name: Signal::new(String::new()),
            new_description: Signal::new(String::new()),
            season: SeasonState::new(None),
        }
    }

    /// Whether anything was entered — an untouched block in the new-store
    /// form is skipped, not an error.
    pub fn is_filled(&self) -> bool {
        if (self.is_new)() { !self.new_name.read().trim().is_empty() } else { (self.existing)().is_some() }
    }

    pub fn to_input(&self) -> Result<OfferInput, &'static str> {
        let product = if (self.is_new)() {
            ProductChoice::New { name: (self.new_name)(), description: (self.new_description)() }
        } else {
            ProductChoice::Existing((self.existing)().ok_or("error-product-required")?)
        };
        Ok(OfferInput { product, seasonal_months: self.season.value()? })
    }
}

#[component]
pub fn ProductChoiceFields(#[props(into)] id: String, state: OfferState) -> Element {
    let locale = use_locale();
    let catalog = use_catalog();
    let OfferState { is_new, mut existing, mut new_name, mut new_description, .. } = state;
    let selected = use_memo(move || existing());
    rsx! {
        SwitchField { id: "{id}-is-new", label: locale.t("product-form-new-checkbox"), checked: is_new }
        if is_new() {
            Field { label: locale.t("product-form-name"), html_for: "{id}-name",
                Input {
                    id: "{id}-name",
                    value: "{new_name}",
                    oninput: move |e: FormEvent| new_name.set(e.value()),
                }
            }
            Field { label: locale.t("product-form-description-optional"), html_for: "{id}-desc",
                Textarea {
                    id: "{id}-desc",
                    value: "{new_description}",
                    oninput: move |e: FormEvent| new_description.set(e.value()),
                }
            }
        } else {
            Field { label: locale.t("product-form-product"), html_for: "{id}-product",
                Select::<i64> {
                    id: "{id}-product",
                    value: Some(selected.into()),
                    on_value_change: move |v: Option<i64>| existing.set(v),
                    for (i , p) in catalog().into_iter().enumerate() {
                        SelectOption::<i64> {
                            key: "{p.id}",
                            index: i,
                            value: p.id,
                            text_value: format!("{} {}", p.icon_or_default(), p.name),
                            "{p.icon_or_default()} {p.name}"
                        }
                    }
                }
            }
        }
    }
}
