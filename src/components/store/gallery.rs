//! The photo viewer: a full-screen overlay over everything, opened from
//! the header photo or the photo strip, stepping through the shop's
//! photos with the side buttons, the arrow keys or a swipe.

use dioxus::prelude::*;
use dioxus_icons::lucide;

use super::{use_store, Styles};
use crate::i18n::use_locale;

/// How far a pointer has to travel sideways to count as a swipe, in px.
const SWIPE_MIN: f64 = 50.0;

#[component]
pub fn PhotoViewer() -> Element {
    let locale = use_locale();
    let store = use_store();
    let viewing = store.viewing;
    let mut swipe_from = use_signal(|| None::<f64>);

    let d = store.detail.read();
    let count = d.photos.len();
    let Some(index) = viewing().filter(|&i| i < count) else {
        return rsx! {};
    };
    let photo = &d.photos[index];
    let alt = photo.description.clone().unwrap_or_else(|| d.name.clone());

    // Wraps around both ways.
    let go = move |step: isize| {
        let mut viewing = viewing;
        viewing.set(Some((index as isize + step).rem_euclid(count as isize) as usize));
    };
    let close = move || {
        let mut viewing = viewing;
        viewing.set(None);
    };

    rsx! {
        div {
            class: Styles::viewer,
            role: "dialog",
            "aria-modal": "true",
            "aria-label": locale.t("detail-photos"),
            tabindex: "-1",
            // Focus moves in, so the keys below reach it — and the shell's
            // Escape handler, seeing a dialog, leaves the store open.
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e| match e.key() {
                Key::ArrowLeft => go(-1),
                Key::ArrowRight => go(1),
                Key::Escape => {
                    e.prevent_default();
                    close();
                }
                _ => {}
            },
            // The dark backdrop closes it; everything on top stops the click.
            onclick: move |_| close(),
            figure {
                class: Styles::viewer_figure,
                onclick: move |e| e.stop_propagation(),
                onpointerdown: move |e| swipe_from.set(Some(e.client_coordinates().x)),
                onpointerup: move |e| {
                    if let Some(from) = swipe_from.take() {
                        let dx = e.client_coordinates().x - from;
                        if count > 1 && dx.abs() >= SWIPE_MIN {
                            go(if dx < 0.0 { 1 } else { -1 });
                        }
                    }
                },
                img {
                    key: "{photo.id}",
                    class: Styles::viewer_image,
                    src: "/image/{photo.id}",
                    alt: "{alt}",
                    draggable: "false",
                }
                figcaption { class: Styles::viewer_caption,
                    if let Some(caption) = &photo.description {
                        span { "{caption}" }
                    }
                    if count > 1 {
                        span { class: Styles::viewer_counter, "{index + 1} / {count}" }
                    }
                }
            }
            button {
                class: "{Styles::viewer_button} {Styles::viewer_close}",
                r#type: "button",
                title: locale.t("detail-close"),
                "aria-label": locale.t("detail-close"),
                onclick: move |e| {
                    e.stop_propagation();
                    close();
                },
                lucide::X { size: 22 }
            }
            if count > 1 {
                button {
                    class: "{Styles::viewer_button} {Styles::viewer_prev}",
                    r#type: "button",
                    title: locale.t("detail-photo-previous"),
                    "aria-label": locale.t("detail-photo-previous"),
                    onclick: move |e| {
                        e.stop_propagation();
                        go(-1);
                    },
                    lucide::ChevronLeft { size: 26 }
                }
                button {
                    class: "{Styles::viewer_button} {Styles::viewer_next}",
                    r#type: "button",
                    title: locale.t("detail-photo-next"),
                    "aria-label": locale.t("detail-photo-next"),
                    onclick: move |e| {
                        e.stop_propagation();
                        go(1);
                    },
                    lucide::ChevronRight { size: 26 }
                }
            }
        }
    }
}
