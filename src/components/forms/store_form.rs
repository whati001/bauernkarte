//! Creating and editing a store. Its position is picked on the map; a new
//! store also names its first products, since a store with none never
//! shows up in search.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::toast::{use_toast, ToastOptions};

use super::{OfferState, ProductChoiceFields, SeasonFields, SwitchField};
use crate::{
    api::{
        error::AppError,
        store::{create_store, store_for_edit, update_store},
    },
    app::Route,
    components::{
        common::{use_panel_data, Field, FormError, LoadError, Panel, Section},
        account::RequireLogin,
        map::{use_map, Picker},
    },
    i18n::use_locale,
    models::{DayHours, StoreFields, StoreKind},
    opening_hours,
    ui::{
        button::{Button, ButtonVariant},
        input::Input,
        select::{Select, SelectOption},
        textarea::Textarea,
    },
};

/// How many product blocks the new-store form offers. Each appears once
/// the one before it is filled in.
const MAX_OFFERS: usize = 5;

#[component]
pub fn NewStore() -> Element {
    rsx! {
        RequireLogin {
            StoreForm { id: None, initial: StoreFields::default() }
        }
    }
}

#[component]
pub fn EditStore(id: i64) -> Element {
    rsx! {
        RequireLogin {
            EditStoreLoader { id }
        }
    }
}

#[component]
fn EditStoreLoader(id: i64) -> Element {
    // Reactive and non-suspending in the browser — see `use_panel_data`.
    let fields = use_panel_data(use_reactive!(|id| store_for_edit(id)))?;
    match fields {
        Some(Ok(initial)) => rsx! { StoreForm { key: "{id}", id: Some(id), initial } },
        Some(Err(error)) => rsx! { LoadError { error } },
        None => rsx! {},
    }
}

#[component]
fn StoreForm(id: Option<i64>, initial: StoreFields) -> Element {
    let locale = use_locale();
    let mut map = use_map();
    let nav = navigator();
    let toasts = use_toast();

    let mut name = use_signal(|| initial.name.clone());
    let kind = use_signal(|| initial.kind);
    let mut address = use_signal(|| initial.address.clone());
    let mut phone = use_signal(|| initial.phone.clone());
    let mut owner_name = use_signal(|| initial.owner_name.clone());
    let mut owner_since = use_signal(|| initial.owner_since.clone());
    let mut owner_bio = use_signal(|| initial.owner_bio.clone());
    let has_hours = use_signal(|| !initial.openinghours.is_empty());
    let hours = use_signal(|| week_from(&initial.openinghours));
    let offers = use_hook(|| (0..MAX_OFFERS).map(|_| OfferState::new()).collect::<Vec<_>>());
    let mut error = use_signal(|| None::<AppError>);
    let mut saving = use_signal(|| false);

    // The map is the position input while this form is open.
    let start = initial.lat.zip(initial.lon);
    use_effect(move || map.picker.set(Picker { active: true, position: start }));
    use_drop(move || map.picker.set(Picker::default()));
    let position = (map.picker)().position;

    let offers_for_submit = offers.clone();
    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        let offers = offers_for_submit.clone();
        async move {
            let Some((lat, lon)) = (map.picker)().position else {
                error.set(Some(AppError::invalid("error-location-required")));
                return;
            };
            let fields = StoreFields {
                name: name(),
                kind: kind(),
                lat: Some(lat),
                lon: Some(lon),
                openinghours: if has_hours() { week_to_hours(&hours()) } else { Vec::new() },
                address: address(),
                phone: phone(),
                owner_name: owner_name(),
                owner_since: owner_since(),
                owner_bio: owner_bio(),
            };
            saving.set(true);
            let result = match id {
                Some(id) => update_store(id, fields).await.map(|_| None),
                None => {
                    let inputs: Result<Vec<_>, _> =
                        offers.iter().filter(|o| o.is_filled()).map(|o| o.to_input()).collect();
                    match inputs {
                        Ok(inputs) => create_store(fields, inputs).await.map(Some),
                        Err(key) => Err(AppError::invalid(key)),
                    }
                }
            };
            saving.set(false);
            match result {
                Ok(Some(created)) => {
                    toasts.success(locale.t_name("confirmation-pending", &created), ToastOptions::new());
                    nav.push(Route::SearchPanel {});
                }
                Ok(None) => {
                    map.reload_results();
                    nav.push(Route::StorePanel { id: id.unwrap_or_default() });
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    let back = match id {
        Some(id) => Route::StorePanel { id },
        None => Route::SearchPanel {},
    };
    let title = if id.is_some() { locale.t("store-form-edit-heading") } else { locale.t("store-form-new-heading") };

    rsx! {
        Panel { back, title,
            form { class: "form", onsubmit: submit,
                Section { title: locale.t("store-form-name"),
                    Input {
                        id: "store-name",
                        "aria-label": locale.t("store-form-name"),
                        required: true,
                        value: "{name}",
                        oninput: move |e: FormEvent| name.set(e.value()),
                    }
                }
                Section { title: locale.t("store-form-kind"),
                    KindSelect { kind }
                }
                Section { title: locale.t("store-form-location"),
                    p { class: "picker-status",
                        if position.is_some() {
                            {locale.t("search-location-picked")}
                        } else {
                            {locale.t("search-pick-on-map")}
                        }
                    }
                    Button {
                        class: "full-width",
                        r#type: "button",
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            document::eval("window.BK.pickMyLocation();");
                        },
                        lucide::LocateFixed { size: 16 }
                        {locale.t("search-use-my-location")}
                    }
                }
                Section { title: locale.t("store-form-contact-heading"),
                    Field { label: locale.t("store-form-address"), html_for: "store-address",
                        Input {
                            id: "store-address",
                            autocomplete: "street-address",
                            value: "{address}",
                            oninput: move |e: FormEvent| address.set(e.value()),
                        }
                    }
                    Field { label: locale.t("store-form-phone"), html_for: "store-phone",
                        Input {
                            id: "store-phone",
                            r#type: "tel",
                            value: "{phone}",
                            oninput: move |e: FormEvent| phone.set(e.value()),
                        }
                    }
                }
                Section { title: locale.t("store-form-owner-heading"),
                    Field { label: locale.t("store-form-owner-name"), html_for: "owner-name",
                        Input { id: "owner-name", value: "{owner_name}", oninput: move |e: FormEvent| owner_name.set(e.value()) }
                    }
                    Field { label: locale.t("store-form-owner-since"), html_for: "owner-since",
                        Input {
                            id: "owner-since",
                            inputmode: "numeric",
                            maxlength: 4,
                            value: "{owner_since}",
                            oninput: move |e: FormEvent| owner_since.set(e.value()),
                        }
                    }
                    Field { label: locale.t("store-form-owner-bio"), html_for: "owner-bio",
                        Textarea { id: "owner-bio", value: "{owner_bio}", oninput: move |e: FormEvent| owner_bio.set(e.value()) }
                    }
                }
                Section { title: locale.t("store-form-opening-hours"),
                    SwitchField { id: "has-hours", label: locale.t("store-form-define-opening-hours"), checked: has_hours }
                    if has_hours() {
                        p { class: "field-hint", {locale.t("store-form-opening-hours-hint")} }
                        HoursFields { hours }
                    }
                }
                if id.is_none() {
                    Section { title: locale.t("store-form-product-heading"),
                        for (i , offer) in offers.iter().copied().enumerate() {
                            // A block shows once every block before it is filled.
                            if offers[..i].iter().all(|o| o.is_filled()) {
                                div { key: "{i}", class: "offer-block",
                                    h4 { "{locale.t(\"store-form-product-n\")} {i + 1}" }
                                    ProductChoiceFields { id: "offer-{i}", state: offer }
                                    SeasonFields { id: "offer-{i}", state: offer.season }
                                }
                            }
                        }
                    }
                }
                FormError { error }
                Button { r#type: "submit", disabled: saving() || position.is_none(),
                    lucide::CircleCheck { size: 16 }
                    {locale.t("action-save")}
                }
            }
        }
    }
}

/// Monday..Sunday as (open, close), `""` for closed.
fn week_from(hours: &[DayHours]) -> [(String, String); 7] {
    std::array::from_fn(|i| {
        hours
            .iter()
            .find(|h| h.day == i as i16 + 1)
            .map(|h| (h.open.clone(), h.close.clone()))
            .unwrap_or_default()
    })
}

fn week_to_hours(week: &[(String, String); 7]) -> Vec<DayHours> {
    week.iter()
        .enumerate()
        .filter(|(_, (open, close))| !open.is_empty() || !close.is_empty())
        .map(|(i, (open, close))| DayHours { day: i as i16 + 1, open: open.clone(), close: close.clone() })
        .collect()
}

#[component]
fn HoursFields(hours: Signal<[(String, String); 7]>) -> Element {
    let locale = use_locale();
    let times = opening_hours::time_options();
    rsx! {
        div { class: "hours-grid",
            for (i , (day , label_key)) in opening_hours::WEEKDAYS.iter().enumerate() {
                div { key: "{day}", class: "hours-row",
                    span { class: "hours-day", {locale.t(label_key)} }
                    TimeSelect {
                        label: format!("{} {}", locale.t(label_key), locale.t("store-form-opens")),
                        value: hours.read()[i].0.clone(),
                        times: times.clone(),
                        on_change: move |v: String| hours.write()[i].0 = v,
                    }
                    span { "–" }
                    TimeSelect {
                        label: format!("{} {}", locale.t(label_key), locale.t("store-form-closes")),
                        value: hours.read()[i].1.clone(),
                        times: times.clone(),
                        on_change: move |v: String| hours.write()[i].1 = v,
                    }
                }
            }
        }
    }
}

/// Market, vending machine or shop — what the map pin shows.
#[component]
fn KindSelect(kind: Signal<StoreKind>) -> Element {
    let locale = use_locale();
    let current = use_memo(move || Some(kind()));
    rsx! {
        Select::<StoreKind> {
            class: "full-width",
            "aria-label": locale.t("store-form-kind"),
            value: Some(current.into()),
            on_value_change: move |v: Option<StoreKind>| kind.set(v.unwrap_or_default()),
            for (i , option) in StoreKind::ALL.into_iter().enumerate() {
                SelectOption::<StoreKind> {
                    key: "{option.as_str()}",
                    index: i,
                    value: option,
                    text_value: locale.t(option.label_key()),
                    {locale.t(option.label_key())}
                }
            }
        }
    }
}

/// Half-hour steps up to "24:00", plus "closed" (the empty value).
#[component]
fn TimeSelect(label: String, value: String, times: Vec<String>, on_change: EventHandler<String>) -> Element {
    let locale = use_locale();
    let current = use_memo(use_reactive!(|value| Some(value)));
    rsx! {
        Select::<String> {
            class: "time-select",
            "aria-label": "{label}",
            value: Some(current.into()),
            on_value_change: move |v: Option<String>| on_change.call(v.unwrap_or_default()),
            SelectOption::<String> { index: 0usize, value: String::new(), text_value: locale.t("opening-hours-closed"),
                {locale.t("opening-hours-closed")}
            }
            for (i , t) in times.into_iter().enumerate() {
                SelectOption::<String> { key: "{t}", index: i + 1, value: t.clone(), text_value: t.clone(), "{t}" }
            }
        }
    }
}
