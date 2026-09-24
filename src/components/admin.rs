//! The admin area: moderation queues, accounts and the Impressum's
//! contents. Full-page (no map); every server function behind it 404s for
//! anyone but an admin, and so does this UI.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::toast::{use_toast, ToastOptions};

use crate::{
    api::{
        admin::{
            admin_create_user, admin_delete_image, admin_delete_product, admin_delete_user, admin_moderate,
            admin_queue, admin_rail, admin_revert, admin_save_site_info, admin_set_admin, admin_site_info,
            admin_update_image, admin_update_product, admin_users,
        },
        error::AppError,
    },
    app::{use_session, Route},
    components::{
        common::{Field, FormError},
        navbar::Navbar,
    },
    i18n::use_locale,
    models::{AdminImageRow, AdminProductRow, AdminUserRow, Entity, QueuePage, QueueTab, SiteInfo, DEFAULT_PRODUCT_ICON, PRODUCT_ICONS},
    ui::{
        alert_dialog::{AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogDescription, AlertDialogTitle},
        badge::{Badge, BadgeVariant},
        button::{Button, ButtonSize, ButtonVariant},
        card::{Card, CardContent, CardHeader, CardTitle},
        checkbox::Checkbox,
        input::Input,
        label::Label,
        tabs::{TabContent, TabList, TabTrigger, Tabs},
        textarea::Textarea,
    },
};

/// Bumped after any admin action, so the rail's badges catch up.
#[derive(Clone, Copy)]
struct RailRefresh(Signal<u32>);

fn entity_icon(entity: Entity) -> Element {
    match entity {
        Entity::Store => rsx! { lucide::Store { size: 16 } },
        Entity::Product => rsx! { lucide::Package { size: 16 } },
        Entity::Offer => rsx! { lucide::Tag { size: 16 } },
        Entity::Image => rsx! { lucide::Image { size: 16 } },
    }
}

#[component]
pub fn AdminShell() -> Element {
    let locale = use_locale();
    let session = use_session();
    let refresh = use_context_provider(|| RailRefresh(Signal::new(0))).0;
    let rail = use_resource(move || {
        let _ = refresh();
        admin_rail()
    });
    let route = use_route::<Route>();

    if !session.user().is_some_and(|u| u.admin) {
        return rsx! {
            div { class: "not-found",
                h1 { "404" }
                Link { to: Route::SearchPanel {}, {locale.t("admin-back-to-map")} }
            }
        };
    }
    let counts = rail().and_then(Result::ok).unwrap_or_default();
    let current_slug = match &route {
        Route::AdminQueue { slug, .. } => Some(slug.clone()),
        _ => None,
    };

    rsx! {
        document::Title { {locale.t("admin-title")} }
        div { class: "app admin",
            Navbar { with_search: false }
            div { class: "admin-shell",
                nav { class: "admin-rail", "aria-label": locale.t("admin-nav-label"),
                    Link { class: "back-link", to: Route::SearchPanel {},
                        lucide::ChevronLeft { size: 16 }
                        {locale.t("admin-back-to-map")}
                    }
                    span { class: "admin-rail-title", {locale.t("admin-title")} }
                    Link { class: "admin-rail-item", to: Route::AdminUsers {}, active_class: "active",
                        lucide::Users { size: 16 }
                        {locale.t("admin-nav-users")}
                    }
                    for entity in Entity::ALL {
                        Link {
                            key: "{entity.slug()}",
                            class: if current_slug.as_deref() == Some(entity.slug()) { "admin-rail-item active" } else { "admin-rail-item" },
                            to: Route::AdminQueue { slug: entity.slug().into(), tab: "pending".into() },
                            {entity_icon(entity)}
                            {locale.t(entity.label_key())}
                            if let Some(n) = counts.iter().find(|c| c.entity == entity).map(|c| c.pending).filter(|n| *n > 0) {
                                Badge { class: "admin-count", "{n}" }
                            }
                        }
                    }
                    div { class: "admin-rail-sep" }
                    Link { class: "admin-rail-item", to: Route::AdminSiteInfo {}, active_class: "active",
                        lucide::Building2 { size: 16 }
                        {locale.t("admin-nav-site-info")}
                    }
                }
                main { class: "admin-content",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

/// `/admin`: the first queue with work waiting, or the users page.
#[component]
pub fn AdminIndex() -> Element {
    let nav = navigator();
    let rail = use_resource(admin_rail);
    use_effect(move || {
        if let Some(Ok(counts)) = &*rail.read() {
            let target = counts
                .iter()
                .find(|c| c.pending > 0)
                .map(|c| Route::AdminQueue { slug: c.entity.slug().into(), tab: "pending".into() })
                .unwrap_or(Route::AdminUsers {});
            nav.replace(target);
        }
    });
    rsx! {}
}

#[component]
pub fn AdminQueue(slug: String, tab: String) -> Element {
    let locale = use_locale();
    let nav = navigator();
    let Some(entity) = Entity::from_slug(&slug) else {
        return rsx! { p { class: "admin-empty", {locale.t("error-not-found")} } };
    };
    let tabs = QueueTab::for_entity(entity);
    let tab = Some(QueueTab::from_key(&tab)).filter(|t| tabs.contains(t)).unwrap_or(QueueTab::Pending);
    let mut rail_refresh = use_context::<RailRefresh>().0;
    let mut page = use_resource(use_reactive!(|slug, tab| async move { admin_queue(slug, tab.key().to_string()).await }));
    let mut error = use_signal(|| None::<AppError>);

    let act = move |run: std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>>>>| {
        spawn(async move {
            match run.await {
                Ok(()) => error.set(None),
                Err(err) => error.set(Some(err)),
            }
            page.restart();
            *rail_refresh.write() += 1;
        });
    };

    let data: Option<QueuePage> = page().and_then(Result::ok);
    let counts = data.as_ref().map(|d| d.counts.clone()).unwrap_or_default();
    let slug_for_tabs = slug.clone();

    rsx! {
        div { class: "admin-head",
            h2 { {locale.t(entity.label_key())} }
            p { {locale.t(entity.blurb_key())} }
        }
        FormError { error }
        Tabs {
            value: Some(tab.key().to_string()),
            on_value_change: move |key: String| {
                nav.replace(Route::AdminQueue { slug: slug_for_tabs.clone(), tab: key });
            },
            horizontal: true,
            TabList {
                for (i , t) in tabs.iter().copied().enumerate() {
                    TabTrigger { key: "{t.key()}", value: t.key().to_string(), index: i,
                        {locale.t(t.label_key())}
                        " "
                        Badge { variant: BadgeVariant::Secondary, "{counts.for_tab(t)}" }
                    }
                }
            }
            for (i , t) in tabs.iter().copied().enumerate() {
                TabContent { key: "{t.key()}", value: t.key().to_string(), index: i,
                    if t == tab {
                        match &data {
                            None => rsx! {},
                            Some(d) if d.rows.is_empty() && d.changes.is_empty() && d.products.is_empty() && d.images.is_empty() => rsx! {
                                p { class: "admin-empty", {locale.t("admin-queue-empty")} }
                            },
                            Some(d) => rsx! {
                                div { class: "admin-rows",
                                    for product in d.products.clone() {
                                        ProductRow {
                                            key: "{product.id}",
                                            product,
                                            on_change: move |_| {
                                                page.restart();
                                                *rail_refresh.write() += 1;
                                            },
                                        }
                                    }
                                    for image in d.images.clone() {
                                        ImageRow {
                                            key: "{image.id}",
                                            image,
                                            on_change: move |_| {
                                                page.restart();
                                                *rail_refresh.write() += 1;
                                            },
                                        }
                                    }
                                    for row in d.rows.clone() {
                                        article { key: "{row.id}", class: "admin-row",
                                            div { class: "admin-row-main",
                                                div { class: "admin-row-title",
                                                    strong { "{row.title}" }
                                                    if tab == QueueTab::Pending { Badge { {locale.t("admin-pill-new")} } }
                                                    if tab == QueueTab::Deleted { Badge { variant: BadgeVariant::Destructive, {locale.t("admin-pill-deleted")} } }
                                                    if row.store_image { Badge { variant: BadgeVariant::Outline, {locale.t("admin-image-cover")} } }
                                                }
                                                div { class: "admin-row-meta",
                                                    if let Some(s) = &row.subtitle { span { "{s}" } }
                                                    if let Some(a) = &row.author { span { "{a}" } }
                                                    time { datetime: "{row.at_iso}", "{row.at_human}" }
                                                }
                                            }
                                            div { class: "admin-row-actions",
                                                if let (Entity::Store, true) = (entity, tab != QueueTab::Deleted) {
                                                    Link { class: "button-link small", to: Route::EditStore { id: row.id }, {locale.t("admin-action-view")} }
                                                }
                                                if entity == Entity::Image {
                                                    a { class: "button-link small", href: "/image/{row.id}", target: "_blank", {locale.t("admin-action-view")} }
                                                }
                                                if tab == QueueTab::Pending {
                                                    Button {
                                                        size: ButtonSize::Sm,
                                                        variant: ButtonVariant::Outline,
                                                        onclick: {
                                                            let slug = slug.clone();
                                                            move |_| act(Box::pin(admin_moderate(slug.clone(), row.id, "reject".into())))
                                                        },
                                                        {locale.t("admin-action-reject")}
                                                    }
                                                    Button {
                                                        size: ButtonSize::Sm,
                                                        onclick: {
                                                            let slug = slug.clone();
                                                            move |_| act(Box::pin(admin_moderate(slug.clone(), row.id, "approve".into())))
                                                        },
                                                        {locale.t("admin-action-approve")}
                                                    }
                                                }
                                                if tab == QueueTab::Deleted {
                                                    Button {
                                                        size: ButtonSize::Sm,
                                                        onclick: {
                                                            let slug = slug.clone();
                                                            move |_| act(Box::pin(admin_moderate(slug.clone(), row.id, "restore".into())))
                                                        },
                                                        {locale.t("admin-action-restore")}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    for change in d.changes.clone() {
                                        article { key: "{change.log_id}", class: "admin-row",
                                            div { class: "admin-row-main",
                                                div { class: "admin-row-title",
                                                    strong { "{change.title}" }
                                                    Badge { variant: BadgeVariant::Outline, {locale.t("admin-pill-edited")} }
                                                }
                                                div { class: "admin-row-meta",
                                                    if let Some(a) = &change.author { span { "{a}" } }
                                                    time { datetime: "{change.at_iso}", "{change.at_human}" }
                                                }
                                                if !change.diff.is_empty() {
                                                    div { class: "admin-diff",
                                                        for d in change.diff.iter() {
                                                            div { class: "admin-diff-row",
                                                                span { class: "admin-diff-field", "{d.field}" }
                                                                span { class: "admin-diff-old", "{d.old}" }
                                                                span { "aria-hidden": "true", "→" }
                                                                span { class: "admin-diff-new", "{d.new}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "admin-row-actions",
                                                Button {
                                                    size: ButtonSize::Sm,
                                                    variant: ButtonVariant::Outline,
                                                    onclick: {
                                                        let slug = slug.clone();
                                                        move |_| act(Box::pin(admin_revert(slug.clone(), change.log_id)))
                                                    },
                                                    {locale.t("admin-action-revert")}
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// One live product on the "existing" tab: edit in place, or delete it
/// (which takes it off every store too).
#[component]
fn ProductRow(product: AdminProductRow, on_change: EventHandler<()>) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let mut editing = use_signal(|| false);
    let mut confirm = use_signal(|| false);
    let id = product.id;
    let icon = product.icon.clone().unwrap_or_else(|| DEFAULT_PRODUCT_ICON.into());

    rsx! {
        article { class: "admin-row admin-product",
            div { class: "admin-product-head",
                span { class: "admin-product-icon", "aria-hidden": "true", "{icon}" }
                div { class: "admin-row-main",
                    div { class: "admin-row-title", strong { "{product.name}" } }
                    div { class: "admin-row-meta",
                        if let Some(d) = &product.description { span { "{d}" } }
                        span { {locale.t_count("admin-product-stores", product.stores)} }
                    }
                }
                div { class: "admin-row-actions",
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        "aria-expanded": "{editing}",
                        onclick: move |_| editing.toggle(),
                        lucide::Pencil { size: 14 }
                        {locale.t("admin-action-edit")}
                    }
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| confirm.set(true),
                        lucide::Trash2 { size: 14 }
                        {locale.t("action-delete")}
                    }
                    AlertDialog { open: confirm(), on_open_change: move |v| confirm.set(v),
                        AlertDialogTitle { {locale.t("action-delete")} ": {product.name}" }
                        AlertDialogDescription { {locale.t_count("admin-product-delete-warning", product.stores)} }
                        AlertDialogActions {
                            AlertDialogCancel { {locale.t("action-cancel")} }
                            AlertDialogAction {
                                on_click: move |_| async move {
                                    match admin_delete_product(id).await {
                                        Ok(()) => toasts.success(locale.t("admin-product-deleted"), ToastOptions::new()),
                                        Err(err) => toasts.error(locale.t_error(&err.key), ToastOptions::new()),
                                    }
                                    on_change.call(());
                                },
                                {locale.t("action-delete")}
                            }
                        }
                    }
                }
            }
            if editing() {
                ProductEditForm {
                    product: product.clone(),
                    on_done: move |saved: bool| {
                        editing.set(false);
                        if saved {
                            on_change.call(());
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ProductEditForm(product: AdminProductRow, on_done: EventHandler<bool>) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let id = product.id;
    let mut name = use_signal(|| product.name.clone());
    let mut description = use_signal(|| product.description.clone().unwrap_or_default());
    let mut icon = use_signal(|| product.icon.clone().unwrap_or_default());
    let mut error = use_signal(|| None::<AppError>);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match admin_update_product(id, name(), description(), icon()).await {
                Ok(()) => {
                    toasts.success(locale.t_name("confirmation-updated", &name()), ToastOptions::new());
                    on_done.call(true);
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        form { class: "form admin-product-form", onsubmit: submit,
            Field { label: locale.t("edit-product-form-name"), html_for: "product-{id}-name",
                Input { id: "product-{id}-name", required: true, value: "{name}", oninput: move |e: FormEvent| name.set(e.value()) }
            }
            Field { label: locale.t("edit-product-form-description"), html_for: "product-{id}-description",
                Textarea {
                    id: "product-{id}-description",
                    rows: 2,
                    value: "{description}",
                    oninput: move |e: FormEvent| description.set(e.value()),
                }
            }
            Field { label: locale.t("admin-product-icon"), html_for: "product-{id}-icon",
                div { class: "icon-picker", role: "group", "aria-label": locale.t("admin-product-icon"),
                    for choice in PRODUCT_ICONS.iter().copied() {
                        button {
                            key: "{choice}",
                            r#type: "button",
                            class: if icon() == choice { "icon-choice selected" } else { "icon-choice" },
                            "aria-pressed": "{icon() == choice}",
                            onclick: move |_| icon.set(choice.to_string()),
                            "{choice}"
                        }
                    }
                }
                Input {
                    id: "product-{id}-icon",
                    value: "{icon}",
                    placeholder: DEFAULT_PRODUCT_ICON,
                    oninput: move |e: FormEvent| icon.set(e.value()),
                }
                p { class: "field-hint", {locale.t("admin-product-icon-hint")} }
            }
            FormError { error }
            div { class: "form-actions",
                Button { r#type: "submit",
                    lucide::CircleCheck { size: 16 }
                    {locale.t("action-save")}
                }
                Button { r#type: "button", variant: ButtonVariant::Outline, onclick: move |_| on_done.call(false),
                    {locale.t("action-cancel")}
                }
            }
        }
    }
}

/// One live image on the "existing" tab: a thumbnail, its store and
/// flags; edit in place, or delete it (off the store for good, until
/// restored).
#[component]
fn ImageRow(image: AdminImageRow, on_change: EventHandler<()>) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let mut editing = use_signal(|| false);
    let mut confirm = use_signal(|| false);
    let id = image.id;

    rsx! {
        article { class: "admin-row admin-product",
            div { class: "admin-product-head",
                a { class: "admin-thumb", href: "/image/{id}", target: "_blank", rel: "noopener",
                    img { src: "/image/{id}", alt: image.description.clone().unwrap_or_default(), loading: "lazy" }
                }
                div { class: "admin-row-main",
                    div { class: "admin-row-title",
                        Link { to: Route::StorePanel { id: image.store_id }, strong { "{image.store_name}" } }
                        if image.cover { Badge { {locale.t("admin-image-cover")} } }
                        if image.is_owner { Badge { variant: BadgeVariant::Outline, {locale.t("admin-image-owner")} } }
                    }
                    div { class: "admin-row-meta",
                        if let Some(d) = &image.description { span { "{d}" } }
                    }
                }
                div { class: "admin-row-actions",
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        "aria-expanded": "{editing}",
                        onclick: move |_| editing.toggle(),
                        lucide::Pencil { size: 14 }
                        {locale.t("admin-action-edit")}
                    }
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| confirm.set(true),
                        lucide::Trash2 { size: 14 }
                        {locale.t("action-delete")}
                    }
                    AlertDialog { open: confirm(), on_open_change: move |v| confirm.set(v),
                        AlertDialogTitle { {locale.t("action-delete")} ": {image.store_name}" }
                        AlertDialogDescription { {locale.t("admin-image-delete-warning")} }
                        AlertDialogActions {
                            AlertDialogCancel { {locale.t("action-cancel")} }
                            AlertDialogAction {
                                on_click: move |_| async move {
                                    match admin_delete_image(id).await {
                                        Ok(()) => toasts.success(locale.t("admin-image-deleted"), ToastOptions::new()),
                                        Err(err) => toasts.error(locale.t_error(&err.key), ToastOptions::new()),
                                    }
                                    on_change.call(());
                                },
                                {locale.t("action-delete")}
                            }
                        }
                    }
                }
            }
            if editing() {
                ImageEditForm {
                    image: image.clone(),
                    on_done: move |saved: bool| {
                        editing.set(false);
                        if saved {
                            on_change.call(());
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ImageEditForm(image: AdminImageRow, on_done: EventHandler<bool>) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let id = image.id;
    let mut description = use_signal(|| image.description.clone().unwrap_or_default());
    let mut cover = use_signal(|| image.cover);
    let mut error = use_signal(|| None::<AppError>);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match admin_update_image(id, description(), cover()).await {
                Ok(()) => {
                    toasts.success(locale.t("admin-image-saved"), ToastOptions::new());
                    on_done.call(true);
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        form { class: "form admin-product-form", onsubmit: submit,
            Field { label: locale.t("edit-product-form-description"), html_for: "image-{id}-description",
                Input { id: "image-{id}-description", value: "{description}", oninput: move |e: FormEvent| description.set(e.value()) }
            }
            // A portrait belongs to the owner block, never the header.
            if !image.is_owner {
                div { class: "switch-field",
                    Checkbox {
                        id: "image-{id}-cover",
                        checked: Some(if cover() { dioxus_primitives::checkbox::CheckboxState::Checked } else { dioxus_primitives::checkbox::CheckboxState::Unchecked }),
                        on_checked_change: move |s| cover.set(s == dioxus_primitives::checkbox::CheckboxState::Checked),
                    }
                    Label { html_for: "image-{id}-cover", {locale.t("image-form-is-cover")} }
                }
            }
            FormError { error }
            div { class: "form-actions",
                Button { r#type: "submit",
                    lucide::CircleCheck { size: 16 }
                    {locale.t("action-save")}
                }
                Button { r#type: "button", variant: ButtonVariant::Outline, onclick: move |_| on_done.call(false),
                    {locale.t("action-cancel")}
                }
            }
        }
    }
}

#[component]
pub fn AdminUsers() -> Element {
    let locale = use_locale();
    let mut users = use_resource(admin_users);
    let rows: Vec<AdminUserRow> = users().and_then(Result::ok).unwrap_or_default();

    rsx! {
        div { class: "admin-head",
            h2 { {locale.t("admin-users-heading")} }
            p { {locale.t("admin-users-blurb")} }
        }
        NewUserCard { on_created: move |_| users.restart() }
        div { class: "admin-table-wrap",
            table { class: "admin-table",
                thead {
                    tr {
                        th { scope: "col", {locale.t("auth-name")} }
                        th { scope: "col", {locale.t("admin-users-role")} }
                        th { scope: "col", class: "num", {locale.t("admin-users-contributions")} }
                        th { scope: "col", class: "num", {locale.t("admin-users-registered")} }
                        th { scope: "col", span { class: "sr-only", {locale.t("admin-users-actions")} } }
                    }
                }
                tbody {
                    for user in rows {
                        UserRow { key: "{user.id}", user, on_change: move |_| users.restart() }
                    }
                }
            }
        }
    }
}

#[component]
fn UserRow(user: AdminUserRow, on_change: EventHandler<()>) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let mut confirm = use_signal(|| false);
    let id = user.id;
    let make_admin = !user.admin;

    let report = move |result: Result<(), AppError>, ok_key: &'static str| {
        match result {
            Ok(()) => toasts.success(locale.t(ok_key), ToastOptions::new()),
            Err(err) => toasts.error(locale.t_error(&err.key), ToastOptions::new()),
        }
        on_change.call(());
    };

    rsx! {
        tr {
            td {
                div { class: "admin-user-name", "{user.name}" }
                div { class: "admin-user-mail", "{user.email}" }
            }
            td {
                if user.admin {
                    Badge { {locale.t("admin-users-admin")} }
                } else {
                    Badge { variant: BadgeVariant::Outline, {locale.t("admin-users-member")} }
                }
            }
            td { class: "num", "{user.contributions}" }
            td { class: "num", "{user.created_human}" }
            td { class: "acts",
                if !user.protected {
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        onclick: move |_| async move { report(admin_set_admin(id, make_admin).await, "admin-users-role-changed") },
                        if user.admin { {locale.t("admin-users-revoke-admin")} } else { {locale.t("admin-users-grant-admin")} }
                    }
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| confirm.set(true),
                        {locale.t("admin-users-delete")}
                    }
                    // A real DELETE, the one irreversible action here.
                    AlertDialog { open: confirm(), on_open_change: move |v| confirm.set(v),
                        AlertDialogTitle { {locale.t("admin-users-delete")} ": {user.name}" }
                        AlertDialogDescription { {locale.t_count("admin-users-delete-warning", user.contributions)} }
                        AlertDialogActions {
                            AlertDialogCancel { {locale.t("action-cancel")} }
                            AlertDialogAction {
                                on_click: move |_| async move { report(admin_delete_user(id).await, "admin-users-deleted") },
                                {locale.t("admin-users-delete-confirm")}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NewUserCard(on_created: EventHandler<()>) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut admin = use_signal(|| false);
    let mut error = use_signal(|| None::<AppError>);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match admin_create_user(name(), email(), password(), admin()).await {
                Ok(()) => {
                    error.set(None);
                    name.set(String::new());
                    email.set(String::new());
                    password.set(String::new());
                    admin.set(false);
                    toasts.success(locale.t("admin-users-created"), ToastOptions::new());
                    on_created.call(());
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        Card { class: "admin-card",
            CardHeader {
                CardTitle { {locale.t("admin-users-new")} }
            }
            CardContent {
                form { class: "admin-inline-form", onsubmit: submit,
                    Field { label: locale.t("auth-name"), html_for: "new-user-name",
                        Input { id: "new-user-name", required: true, value: "{name}", oninput: move |e: FormEvent| name.set(e.value()) }
                    }
                    Field { label: locale.t("auth-email"), html_for: "new-user-email",
                        Input { id: "new-user-email", r#type: "email", required: true, value: "{email}", oninput: move |e: FormEvent| email.set(e.value()) }
                    }
                    Field { label: locale.t("auth-password"), html_for: "new-user-password",
                        Input {
                            id: "new-user-password",
                            r#type: "password",
                            autocomplete: "new-password",
                            required: true,
                            value: "{password}",
                            oninput: move |e: FormEvent| password.set(e.value()),
                        }
                    }
                    div { class: "switch-field",
                        Checkbox {
                            id: "new-user-admin",
                            checked: Some(if admin() { dioxus_primitives::checkbox::CheckboxState::Checked } else { dioxus_primitives::checkbox::CheckboxState::Unchecked }),
                            on_checked_change: move |s| admin.set(s == dioxus_primitives::checkbox::CheckboxState::Checked),
                        }
                        Label { html_for: "new-user-admin", {locale.t("admin-users-is-admin")} }
                    }
                    Button { r#type: "submit", {locale.t("admin-users-create")} }
                }
                FormError { error }
            }
        }
    }
}

#[component]
pub fn AdminSiteInfo() -> Element {
    let info = use_resource(admin_site_info);
    match info() {
        Some(Ok(info)) => rsx! { SiteInfoForm { info } },
        _ => rsx! {},
    }
}

#[component]
fn SiteInfoForm(info: SiteInfo) -> Element {
    let locale = use_locale();
    let toasts = use_toast();
    let mut form = use_signal(|| info);
    let mut error = use_signal(|| None::<AppError>);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match admin_save_site_info(form()).await {
                Ok(()) => {
                    error.set(None);
                    toasts.success(locale.t("admin-site-info-saved"), ToastOptions::new());
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    // Every field is optional free text; one input per `SiteInfo` field.
    let text_field = move |id: &'static str, label_key: &'static str, get: fn(&SiteInfo) -> &String, set: fn(&mut SiteInfo, String)| {
        rsx! {
            Field { label: locale.t(label_key), html_for: id,
                Input { id, value: get(&form.read()).clone(), oninput: move |e: FormEvent| set(&mut form.write(), e.value()) }
            }
        }
    };

    rsx! {
        div { class: "admin-head",
            h2 { {locale.t("admin-nav-site-info")} }
            p { {locale.t("admin-site-info-blurb")} }
        }
        form { class: "form admin-form", onsubmit: submit,
            Card {
                CardHeader { CardTitle { {locale.t("impressum-operator")} } }
                CardContent {
                    {text_field("si-operator", "admin-site-info-operator", |s| &s.operator_name, |s, v| s.operator_name = v)}
                    {text_field("si-street", "admin-site-info-street", |s| &s.street, |s, v| s.street = v)}
                    div { class: "field-row",
                        {text_field("si-postal", "admin-site-info-postal", |s| &s.postal_code, |s, v| s.postal_code = v)}
                        {text_field("si-city", "admin-site-info-city", |s| &s.city, |s, v| s.city = v)}
                    }
                    {text_field("si-country", "admin-site-info-country", |s| &s.country, |s, v| s.country = v)}
                }
            }
            Card {
                CardHeader { CardTitle { {locale.t("impressum-contact")} } }
                CardContent {
                    {text_field("si-email", "auth-email", |s| &s.email, |s, v| s.email = v)}
                    {text_field("si-phone", "impressum-phone", |s| &s.phone, |s, v| s.phone = v)}
                }
            }
            Card {
                CardHeader { CardTitle { {locale.t("impressum-legal")} } }
                CardContent {
                    p { class: "field-hint", {locale.t("admin-site-info-optional")} }
                    {text_field("si-vat", "impressum-vat-id", |s| &s.vat_id, |s, v| s.vat_id = v)}
                    {text_field("si-register", "impressum-register", |s| &s.register_number, |s, v| s.register_number = v)}
                    {text_field("si-responsible", "impressum-responsible", |s| &s.responsible, |s, v| s.responsible = v)}
                }
            }
            Card {
                CardHeader { CardTitle { {locale.t("impressum-purpose")} } }
                CardContent {
                    Field { label: locale.t("admin-site-info-purpose"), html_for: "si-purpose",
                        Textarea {
                            id: "si-purpose",
                            rows: 4,
                            value: form.read().purpose.clone(),
                            oninput: move |e: FormEvent| form.write().purpose = e.value(),
                        }
                    }
                }
            }
            FormError { error }
            Button { r#type: "submit",
                lucide::CircleCheck { size: 16 }
                {locale.t("action-save")}
            }
        }
    }
}
