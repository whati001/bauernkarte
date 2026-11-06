//! First-visit guide for the public map application.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

use crate::{
    i18n::use_locale,
    ui::button::{Button, ButtonVariant},
};

const STORAGE_KEY: &str = "bk-onboarding-complete-v1";

#[derive(Clone, Copy)]
struct Page {
    image: &'static str,
    title: &'static str,
    body: &'static str,
}

const PAGES: [Page; 5] = [
    Page {
        image: "search-product",
        title: "onboarding-search-product-title",
        body: "onboarding-search-product-body",
    },
    Page {
        image: "search-store",
        title: "onboarding-search-store-title",
        body: "onboarding-search-store-body",
    },
    Page {
        image: "navbar-login",
        title: "onboarding-navbar-login-title",
        body: "onboarding-navbar-login-body",
    },
    Page {
        image: "add-store",
        title: "onboarding-add-store-title",
        body: "onboarding-add-store-body",
    },
    Page {
        image: "change-store",
        title: "onboarding-change-store-title",
        body: "onboarding-change-store-body",
    },
];

fn complete(mut visible: Signal<bool>) {
    document::eval(&format!(
        "try {{ localStorage.setItem('{STORAGE_KEY}', 'true'); }} catch {{}}"
    ));
    visible.set(false);
}

#[component]
pub fn Onboarding() -> dioxus::prelude::Element {
    let locale = use_locale();
    let mut page_index = use_signal(|| 0usize);
    let mut visible = use_signal(|| false);

    // Start hidden so server rendering and hydration agree. Storage is a
    // browser concern and is checked only after the component has mounted.
    use_effect(move || {
        spawn(async move {
            let result = document::eval(&format!(
                "try {{ return localStorage.getItem('{STORAGE_KEY}') !== 'true'; }} catch {{ return true; }}"
            ))
            .await;
            let should_show = result
                .ok()
                .and_then(|value| value.as_bool())
                .unwrap_or(true);
            if should_show {
                page_index.set(0);
                visible.set(true);
            }
        });
    });

    let index = page_index().min(PAGES.len() - 1);
    let page = PAGES[index];
    let is_last = index + 1 == PAGES.len();
    let step = format!("{} / {}", index + 1, PAGES.len());
    let image = format!("/static/onboarding/{}/{}.png", locale.code(), page.image);

    rsx! {
        DialogRoot {
            class: "onboarding-backdrop",
            open: visible(),
            // Outside clicks and Escape ask the primitive to close. Ignore
            // those requests: only Skip and the final action persist dismissal.
            on_open_change: move |_| {},
            DialogContent { class: "onboarding-card",
                div { class: "onboarding-image",
                    img {
                        src: image,
                        alt: "",
                        width: "1280",
                        height: "800",
                    }
                }
                div { class: "onboarding-copy",
                    div { class: "onboarding-copy-main",
                        DialogTitle { class: "onboarding-title",
                            {locale.t(page.title)}
                        }
                        DialogDescription { class: "onboarding-body",
                            {locale.t(page.body)}
                        }
                    }
                    div {
                        class: "onboarding-progress",
                        "aria-label": step.clone(),
                        span { class: "onboarding-count", "{step}" }
                        span { class: "onboarding-dots", "aria-hidden": "true",
                            for dot in 0..PAGES.len() {
                                span {
                                    class: if dot == index { "onboarding-dot active" } else { "onboarding-dot" }
                                }
                            }
                        }
                    }
                    div { class: "onboarding-actions",
                        Button {
                            class: "onboarding-skip",
                            r#type: "button",
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| complete(visible),
                            {locale.t("onboarding-skip")}
                        }
                        if index > 0 {
                            Button {
                                r#type: "button",
                                variant: ButtonVariant::Outline,
                                onclick: move |_| page_index.set(index - 1),
                                lucide::ArrowLeft { size: 16 }
                                {locale.t("action-back")}
                            }
                        }
                        Button {
                            class: "onboarding-next",
                            r#type: "button",
                            onclick: move |_| {
                                if is_last {
                                    complete(visible);
                                } else {
                                    page_index.set((index + 1).min(PAGES.len() - 1));
                                }
                            },
                            if is_last {
                                {locale.t("onboarding-finish")}
                                lucide::Check { size: 16 }
                            } else {
                                {locale.t("onboarding-next")}
                                lucide::ArrowRight { size: 16 }
                            }
                        }
                    }
                }
            }
        }
    }
}
