//! The header: the shop's first photo, or a drawn farm scene when it has
//! none, with the name set over it and the two ways out pinned on top.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use super::{use_store, Styles};
use crate::{app::Route, components::map::use_map, i18n::use_locale};

/// Foliage hue band for the drawn fallback: yellow-green through green.
/// A free 0–360° hue gave some products magenta fields.
const HUE_BASE: i64 = 72;
const HUE_SPAN: i64 = 58;

#[component]
pub fn StoreHero() -> Element {
    let locale = use_locale();
    let store = use_store();
    let mut viewing = store.viewing;
    let mut map = use_map();
    let nav = navigator();
    let d = store.detail.read();

    // Keyed to the lead product, not the store: every shop leading with
    // apples gets the same scene, which is the point.
    let hue = d.offers.first().map_or(HUE_BASE + 26, |o| HUE_BASE + (o.product_id * 37).rem_euclid(HUE_SPAN));
    let icons: Vec<String> =
        d.offers.iter().take(3).map(|o| o.icon.clone().unwrap_or_else(|| "🌾".into())).collect();

    rsx! {
        // Outside the header, which clips: the bar stays pinned to the top
        // of the panel while the rest of the store scrolls beneath it.
        div { class: Styles::hero_bar,
            Link {
                class: Styles::hero_button.to_string(),
                to: Route::SearchPanel {},
                title: locale.t("action-back-to-search"),
                "aria-label": locale.t("action-back-to-search"),
                lucide::ChevronLeft { size: 18 }
            }
            button {
                class: Styles::hero_button,
                r#type: "button",
                title: locale.t("detail-close"),
                "aria-label": locale.t("detail-close"),
                onclick: move |_| {
                    nav.push(Route::SearchPanel {});
                    map.sidebar_open.set(false);
                },
                lucide::X { size: 18 }
            }
        }
        header { class: Styles::hero,
            if let Some(photo) = d.photos.first() {
                button {
                    class: Styles::hero_open,
                    r#type: "button",
                    title: locale.t("detail-photo-open"),
                    onclick: move |_| viewing.set(Some(0)),
                    img { class: Styles::hero_image, src: "/image/{photo.id}", alt: "{d.name}" }
                }
            } else {
                FarmScene { hue, icons }
            }
            h2 { class: Styles::hero_title, "{d.name}" }
        }
    }
}

/// Sky, sun, rolling hills, a barn, a fenced field and a produce crate —
/// only the land takes the product's hue, or it stops reading as a farm.
#[component]
fn FarmScene(hue: i64, icons: Vec<String>) -> Element {
    let locale = use_locale();
    let land = |s: u8, l: u8| format!("hsl({hue} {s}% {l}%)");
    rsx! {
        div { class: Styles::hero_art, role: "img", "aria-label": locale.t("detail-hero-art-alt"),
            svg {
                view_box: "0 0 400 170",
                preserve_aspect_ratio: "xMidYMid slice",
                "aria-hidden": "true",
                defs {
                    linearGradient { id: "hero-sky", x1: "0", y1: "0", x2: "0", y2: "1",
                        stop { offset: "0%", stop_color: "hsl(199 72% 82%)" }
                        stop { offset: "100%", stop_color: "hsl(186 62% 93%)" }
                    }
                    linearGradient { id: "hero-field", x1: "0", y1: "0", x2: "0", y2: "1",
                        stop { offset: "0%", stop_color: land(40, 52) }
                        stop { offset: "100%", stop_color: land(44, 38) }
                    }
                }
                rect { width: "400", height: "170", fill: "url(#hero-sky)" }
                circle { cx: "330", cy: "38", r: "20", fill: "hsl(46 92% 74%)", opacity: "0.9" }
                path { d: "M0 96 Q 70 66 140 90 T 280 84 T 400 92 L400 170 L0 170 Z", fill: land(32, 68), opacity: "0.8" }
                path { d: "M0 112 Q 90 88 180 108 T 400 104 L400 170 L0 170 Z", fill: land(36, 56), opacity: "0.9" }
                g { transform: "translate(48 62)",
                    path { d: "M0 20 L26 4 L52 20 L52 54 L0 54 Z", fill: "hsl(12 46% 52%)" }
                    path { d: "M-4 21 L26 2 L56 21 L52 25 L26 8 L0 25 Z", fill: "hsl(12 40% 38%)" }
                    rect { x: "18", y: "30", width: "16", height: "24", rx: "1.5", fill: "hsl(30 30% 88%)" }
                    path { d: "M18 30 L34 54 M34 30 L18 54", stroke: "hsl(12 40% 38%)", stroke_width: "2" }
                }
                path { d: "M0 118 Q 200 106 400 118 L400 170 L0 170 Z", fill: "url(#hero-field)" }
                g { stroke: land(40, 38), stroke_width: "1.5", opacity: "0.35", fill: "none",
                    path { d: "M-10 132 Q 200 122 410 132" }
                    path { d: "M-10 146 Q 200 136 410 146" }
                    path { d: "M-10 160 Q 200 150 410 160" }
                }
                g { stroke: "hsl(34 34% 82%)", stroke_width: "3", stroke_linecap: "round", fill: "none",
                    path { d: "M300 122 L300 146 M330 120 L330 144 M360 118 L360 142 M390 116 L390 140" }
                    path { d: "M296 128 Q 345 124 394 122", stroke_width: "2.5" }
                }
                g { transform: "translate(120 112)",
                    path { d: "M0 14 L124 14 L114 46 L10 46 Z", fill: "hsl(34 44% 66%)" }
                    path { d: "M0 14 L124 14 L122 21 L2 21 Z", fill: "hsl(34 40% 56%)" }
                }
            }
            div { class: Styles::hero_produce, "aria-hidden": "true",
                for icon in icons {
                    span { "{icon}" }
                }
            }
        }
    }
}
