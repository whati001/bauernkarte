use dioxus::prelude::*;
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};
use dioxus_icons::lucide::Check;
use dioxus_primitives::checkbox::{self, CheckboxProps};

#[css_module("/src/ui/checkbox/style.css")]
struct Styles;

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    rsx! {
        checkbox::Checkbox {
            checked: props.checked,
            default_checked: props.default_checked,
            required: props.required,
            disabled: props.disabled,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: merge_attributes(vec![attributes!(div { class: Styles::dx_checkbox }), props.attributes]),
            checkbox::CheckboxIndicator { class: Styles::dx_checkbox_indicator,
                Check { size: "1rem" }
            }
        }
    }
}
