use dioxus::prelude::*;
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};
use dioxus_primitives::toggle_group::{self, ToggleGroupProps, ToggleItemProps};

#[css_module("/src/ui/toggle_group/style.css")]
struct Styles;

#[component]
pub fn ToggleGroup(props: ToggleGroupProps) -> Element {
    rsx! {
        toggle_group::ToggleGroup {
            default_pressed: props.default_pressed,
            pressed: props.pressed,
            on_pressed_change: props.on_pressed_change,
            disabled: props.disabled,
            allow_multiple_pressed: props.allow_multiple_pressed,
            horizontal: props.horizontal,
            roving_loop: props.roving_loop,
            attributes: merge_attributes(vec![attributes!(div { class: Styles::dx_toggle_group }), props.attributes]),
            {props.children}
        }
    }
}

#[component]
pub fn ToggleItem(props: ToggleItemProps) -> Element {
    rsx! {
        toggle_group::ToggleItem {
            index: props.index,
            disabled: props.disabled,
            attributes: merge_attributes(vec![attributes!(div { class: Styles::dx_toggle_item }), props.attributes]),
            {props.children}
        }
    }
}
