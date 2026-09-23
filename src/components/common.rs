//! Small building blocks every sidebar panel is made of.

use std::future::Future;

use dioxus::{fullstack::Transportable, prelude::*};
use dioxus_icons::lucide;

use crate::{
    api::error::AppError,
    app::Route,
    i18n::use_locale,
    ui::{
        label::Label,
        skeleton::Skeleton,
    },
};

/// A sidebar panel: the way out first (a view's exit is always at the
/// top, never scrolled out of reach), then the title, then the content.
#[component]
pub fn Panel(
    #[props(into, default)] title: Option<String>,
    /// Where the back link goes; `None` hides it.
    #[props(default)]
    back: Option<Route>,
    #[props(into, default)] back_label: Option<String>,
    children: Element,
) -> Element {
    let locale = use_locale();
    rsx! {
        div { class: "panel",
            if let Some(to) = back {
                Link { class: "back-link", to,
                    lucide::ChevronLeft { size: 16 }
                    {back_label.unwrap_or_else(|| locale.t("action-back"))}
                }
            }
            if let Some(title) = title {
                h2 { class: "panel-title", {title} }
            }
            {children}
        }
    }
}

/// A titled group inside a panel.
#[component]
pub fn Section(#[props(into)] title: String, children: Element) -> Element {
    rsx! {
        section { class: "section",
            h3 { class: "section-title", {title} }
            {children}
        }
    }
}

/// A labelled form control.
#[component]
pub fn Field(#[props(into)] label: String, #[props(into)] html_for: String, children: Element) -> Element {
    rsx! {
        div { class: "field",
            Label { html_for, {label} }
            {children}
        }
    }
}

/// The error slot under a form. Server errors arrive as i18n keys.
#[component]
pub fn FormError(error: ReadSignal<Option<AppError>>) -> Element {
    let locale = use_locale();
    rsx! {
        if let Some(err) = error() {
            div { class: "form-error", role: "alert",
                lucide::TriangleAlert { size: 16 }
                {locale.t_error(&err.key)}
            }
        }
    }
}

/// Placeholder while a panel's data loads.
#[component]
pub fn PanelSkeleton() -> Element {
    rsx! {
        div { class: "panel",
            Skeleton { style: "height: 170px; border-radius: 12px;" }
            Skeleton { style: "height: 20px; width: 60%; margin-top: 16px;" }
            Skeleton { style: "height: 16px; width: 80%; margin-top: 10px;" }
            Skeleton { style: "height: 16px; width: 40%; margin-top: 10px;" }
        }
    }
}

/// What a panel shows when its data couldn't be loaded.
#[component]
pub fn LoadError(error: AppError) -> Element {
    let locale = use_locale();
    rsx! {
        Panel { back: Route::SearchPanel {}, back_label: locale.t("action-back-to-search"),
            div { class: "form-error", role: "alert",
                lucide::TriangleAlert { size: 16 }
                {locale.t_error(&error.key)}
            }
        }
    }
}

/// Data a panel loads from the server: `Ok(None)` while it's loading,
/// `Ok(Some(..))` once it's there.
///
/// A `use_server_future` that only suspends on the server, where the
/// render has to wait so a deep link arrives with its data. In the
/// browser, suspending a second time — the panel switching to another
/// store — left the `SuspenseBoundary` showing the old DOM for good, so
/// there it's just "not loaded yet" and the caller shows a placeholder.
#[track_caller]
pub fn use_panel_data<T, F, M>(future: impl FnMut() -> F + 'static) -> Result<Option<T>, RenderError>
where
    F: Future<Output = T> + 'static,
    T: Transportable<M> + Clone,
    M: 'static,
{
    match use_server_future(future) {
        Ok(resource) => Ok(resource()),
        Err(err) if cfg!(feature = "server") => Err(err),
        Err(_) => Ok(None),
    }
}
