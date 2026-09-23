use dioxus::prelude::*;
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};
use dioxus_primitives::switch::{self, SwitchProps};

#[css_module("/src/ui/switch/style.css")]
struct Styles;

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    rsx! {
        switch::Switch {
            checked: props.checked,
            default_checked: props.default_checked,
            disabled: props.disabled,
            required: props.required,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: merge_attributes(vec![attributes!(div { class: Styles::dx_switch }), props.attributes]),
            switch::SwitchThumb { class: Styles::dx_switch_thumb }
        }
    }
}
