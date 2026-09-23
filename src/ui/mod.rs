//! Styled components from the official Dioxus component library
//! (github.com/DioxusLabs/components, built on `dioxus-primitives`),
//! vendored the way `dx components add` does it: each component's source
//! and CSS module live here and are ours to adjust. Colours come from
//! `assets/dx-components-theme.css`, which `assets/theme.css` points at
//! the BauernKarte palette.

pub mod alert_dialog;
pub mod avatar;
pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod combobox;
pub mod dropdown_menu;
pub mod input;
pub mod item;
pub mod label;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod switch;
pub mod tabs;
pub mod textarea;
pub mod toast;
pub mod toggle_group;

use dioxus::prelude::*;

/// Every component stylesheet, linked in the document head.
///
/// `#[css_module]` adds its own link the first time one of its class
/// names is rendered, but only once per process — so on the server only
/// the very first page that used a module carried its stylesheet, and
/// every later request shipped HTML that restyled itself once the WASM
/// booted. Linking them up front makes each server-rendered page arrive
/// complete.
#[component]
pub fn Stylesheets() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/src/ui/alert_dialog/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/avatar/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/badge/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/button/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/card/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/checkbox/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/combobox/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/dropdown_menu/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/input/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/item/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/label/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/select/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/separator/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/skeleton/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/switch/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/tabs/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/textarea/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/toast/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/ui/toggle_group/style.css", AssetOptions::css_module()) }
        document::Stylesheet { href: asset!("/src/components/store/style.css", AssetOptions::css_module()) }
    }
}
