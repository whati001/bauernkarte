use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::dropdown_menu::{
    self, DropdownMenuContentProps, DropdownMenuProps, DropdownMenuTriggerProps,
};
use dioxus_primitives::merge_attributes;

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuItemProps<T: Clone + PartialEq + 'static> {
    pub value: ReadSignal<T>,
    pub index: ReadSignal<usize>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default)]
    pub on_select: Callback<T>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[css_module("/src/ui/dropdown_menu/style.css")]
struct Styles;

#[component]
pub fn DropdownMenu(props: DropdownMenuProps) -> Element {
    let base = attributes!(div {
        class: Styles::dx_dropdown_menu,
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        dropdown_menu::DropdownMenu {
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            disabled: props.disabled,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn DropdownMenuTrigger(props: DropdownMenuTriggerProps) -> Element {
    let base = attributes!(button {
        class: Styles::dx_dropdown_menu_trigger,
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        dropdown_menu::DropdownMenuTrigger { as: props.r#as, attributes: merged, {props.children} }
    }
}

#[component]
pub fn DropdownMenuContent(props: DropdownMenuContentProps) -> Element {
    let base = attributes!(div {
        class: Styles::dx_dropdown_menu_content,
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        dropdown_menu::DropdownMenuContent { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
pub fn DropdownMenuItem<T: Clone + PartialEq + 'static>(
    props: DropdownMenuItemProps<T>,
) -> Element {
    let mut touch_selected = use_signal(|| false);
    let touch_value = props.value;
    let touch_on_select = props.on_select;
    let click_on_select = props.on_select;
    let base = attributes!(div {
        class: Styles::dx_dropdown_menu_item,
        onpointerup: move |event| {
            if event.pointer_type() == "touch" {
                // iOS Safari suppresses the item's synthetic click because the
                // menu content prevents default on pointerdown to keep focus.
                // Select on touch pointerup in Rust, then ignore the later
                // compatibility click on browsers that still emit one.
                event.prevent_default();
                event.stop_propagation();
                touch_selected.set(true);
                touch_on_select.call((touch_value)());
            }
        },
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        dropdown_menu::DropdownMenuItem {
            disabled: props.disabled,
            value: props.value,
            index: props.index,
            on_select: move |value: T| {
                if touch_selected() {
                    touch_selected.set(false);
                    return;
                }
                click_on_select.call(value);
            },
            attributes: merged,
            {props.children}
        }
    }
}
