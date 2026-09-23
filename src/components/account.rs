//! Signing in, registering, and the visitor's own account.

use dioxus::prelude::*;
use dioxus_icons::lucide;
use dioxus_primitives::toast::{use_toast, ToastOptions};

use crate::{
    api::{
        error::AppError,
        session::{account_data, change_password, login, register, update_profile},
    },
    app::{use_session, Route},
    components::common::{Field, FormError, LoadError, Panel, Section},
    credentials::{self, PasswordRule, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH},
    i18n::use_locale,
    models::{AccountData, PendingKind},
    ui::{button::Button, input::Input},
};

/// Renders its children only for a signed-in visitor; everyone else gets
/// a way to sign in instead of a form that would fail on submit.
#[component]
pub fn RequireLogin(children: Element) -> Element {
    let locale = use_locale();
    let session = use_session();
    if session.is_logged_in() {
        return children;
    }
    rsx! {
        Panel { back: Route::SearchPanel {}, back_label: locale.t("action-back-to-search"),
            p { {locale.t("error-login-required")} }
            Link { class: "button-link", to: Route::Login {},
                lucide::LogIn { size: 16 }
                {locale.t("nav-login")}
            }
        }
    }
}

#[component]
pub fn Login() -> Element {
    let locale = use_locale();
    let mut session = use_session();
    let nav = navigator();
    let toasts = use_toast();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<AppError>);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match login(email(), password()).await {
                Ok(user) => {
                    toasts.success(locale.t_name("auth-welcome-back", &user.name), ToastOptions::new());
                    session.0.set(Some(user));
                    nav.push(Route::SearchPanel {});
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        Panel { back: Route::SearchPanel {}, back_label: locale.t("action-back-to-search"), title: locale.t("auth-login-heading"),
            form { class: "form", onsubmit: submit,
                Field { label: locale.t("auth-email"), html_for: "login-email",
                    Input {
                        id: "login-email",
                        r#type: "email",
                        autocomplete: "email",
                        required: true,
                        value: "{email}",
                        oninput: move |e: FormEvent| email.set(e.value()),
                    }
                }
                Field { label: locale.t("auth-password"), html_for: "login-password",
                    Input {
                        id: "login-password",
                        r#type: "password",
                        autocomplete: "current-password",
                        required: true,
                        value: "{password}",
                        oninput: move |e: FormEvent| password.set(e.value()),
                    }
                }
                FormError { error }
                Button { r#type: "submit",
                    lucide::LogIn { size: 16 }
                    {locale.t("auth-login-heading")}
                }
            }
            p { class: "muted",
                {locale.t("auth-no-account")}
                " "
                Link { to: Route::Register {}, {locale.t("auth-register-heading")} }
            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    let locale = use_locale();
    let mut session = use_session();
    let nav = navigator();
    let toasts = use_toast();
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<AppError>);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match register(name(), email(), password()).await {
                Ok(user) => {
                    toasts.success(locale.t_name("auth-register-success", &user.name), ToastOptions::new());
                    session.0.set(Some(user));
                    nav.push(Route::SearchPanel {});
                }
                Err(err) => error.set(Some(err)),
            }
        }
    };

    rsx! {
        Panel { back: Route::Login {}, title: locale.t("auth-register-heading"),
            form { class: "form", onsubmit: submit,
                Field { label: locale.t("auth-name"), html_for: "reg-name",
                    Input { id: "reg-name", autocomplete: "name", required: true, value: "{name}", oninput: move |e: FormEvent| name.set(e.value()) }
                }
                Field { label: locale.t("auth-email"), html_for: "reg-email",
                    Input {
                        id: "reg-email",
                        r#type: "email",
                        autocomplete: "email",
                        required: true,
                        "aria-describedby": "reg-email-policy",
                        value: "{email}",
                        oninput: move |e: FormEvent| email.set(e.value()),
                    }
                    EmailChecklist { id: "reg-email-policy", email }
                }
                Field { label: locale.t("auth-password"), html_for: "reg-password",
                    Input {
                        id: "reg-password",
                        r#type: "password",
                        autocomplete: "new-password",
                        required: true,
                        minlength: MIN_PASSWORD_LENGTH as i64,
                        maxlength: MAX_PASSWORD_LENGTH as i64,
                        "aria-describedby": "reg-password-policy",
                        value: "{password}",
                        oninput: move |e: FormEvent| password.set(e.value()),
                    }
                    PasswordChecklist { id: "reg-password-policy", password, name, email }
                }
                FormError { error }
                Button { r#type: "submit",
                    lucide::UserPlus { size: 16 }
                    {locale.t("auth-register-heading")}
                }
            }
            p { class: "muted",
                {locale.t("auth-have-account")}
                " "
                Link { to: Route::Login {}, {locale.t("auth-login-heading")} }
            }
        }
    }
}

/// One checklist row. An empty field satisfies nothing, or "not a common
/// password" would light up before a key is pressed.
#[component]
fn PolicyRow(met: bool, #[props(into)] label: String) -> Element {
    let locale = use_locale();
    rsx! {
        li { class: if met { "policy-rule met" } else { "policy-rule" },
            if met { lucide::CircleCheck { size: 14 } } else { lucide::Circle { size: 14 } }
            span { {label} }
            span { class: "sr-only",
                if met { {locale.t("policy-rule-met")} } else { {locale.t("policy-rule-unmet")} }
            }
        }
    }
}

/// Runs the same rules the server enforces (`crate::credentials`), live.
#[component]
fn PasswordChecklist(#[props(into)] id: String, password: ReadSignal<String>, name: ReadSignal<String>, email: ReadSignal<String>) -> Element {
    let locale = use_locale();
    let pw = password();
    rsx! {
        ul { class: "policy-list", id: "{id}",
            for rule in PasswordRule::CHECKLIST {
                PolicyRow { met: !pw.is_empty() && rule.is_met(&pw, &name(), &email()), label: locale.t(rule.label_key()) }
            }
        }
    }
}

#[component]
fn EmailChecklist(#[props(into)] id: String, email: ReadSignal<String>) -> Element {
    let locale = use_locale();
    rsx! {
        ul { class: "policy-list", id: "{id}",
            PolicyRow { met: credentials::valid_email(&email()), label: locale.t("email-rule-valid") }
        }
    }
}

#[component]
pub fn Account() -> Element {
    rsx! {
        RequireLogin {
            AccountLoader {}
        }
    }
}

#[component]
fn AccountLoader() -> Element {
    let data = use_server_future(account_data)?;
    match data() {
        Some(Ok(data)) => rsx! { AccountView { data } },
        Some(Err(error)) => rsx! { LoadError { error } },
        None => rsx! {},
    }
}

#[component]
fn AccountView(data: AccountData) -> Element {
    let locale = use_locale();
    let mut session = use_session();
    let toasts = use_toast();
    let mut name = use_signal(|| data.name.clone());
    let mut email = use_signal(|| data.email.clone());
    let mut current = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut profile_error = use_signal(|| None::<AppError>);
    let mut password_error = use_signal(|| None::<AppError>);

    let save_profile = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match update_profile(name(), email()).await {
                Ok(user) => {
                    profile_error.set(None);
                    session.0.set(Some(user));
                    toasts.success(locale.t("account-profile-saved"), ToastOptions::new());
                }
                Err(err) => profile_error.set(Some(err)),
            }
        }
    };
    let save_password = move |evt: FormEvent| {
        evt.prevent_default();
        async move {
            match change_password(current(), new_password()).await {
                Ok(()) => {
                    password_error.set(None);
                    current.set(String::new());
                    new_password.set(String::new());
                    toasts.success(locale.t("account-password-changed"), ToastOptions::new());
                }
                Err(err) => password_error.set(Some(err)),
            }
        }
    };

    rsx! {
        Panel { back: Route::SearchPanel {}, back_label: locale.t("action-back-to-search"), title: locale.t("account-heading"),
            Section { title: locale.t("account-heading"),
                form { class: "form", onsubmit: save_profile,
                    Field { label: locale.t("auth-name"), html_for: "acc-name",
                        Input { id: "acc-name", autocomplete: "name", required: true, value: "{name}", oninput: move |e: FormEvent| name.set(e.value()) }
                    }
                    Field { label: locale.t("auth-email"), html_for: "acc-email",
                        Input {
                            id: "acc-email",
                            r#type: "email",
                            autocomplete: "email",
                            required: true,
                            value: "{email}",
                            oninput: move |e: FormEvent| email.set(e.value()),
                        }
                        EmailChecklist { id: "acc-email-policy", email }
                    }
                    FormError { error: profile_error }
                    Button { r#type: "submit", {locale.t("action-save")} }
                }
            }
            Section { title: locale.t("account-change-password"),
                form { class: "form", onsubmit: save_password,
                    Field { label: locale.t("account-current-password"), html_for: "acc-current",
                        Input {
                            id: "acc-current",
                            r#type: "password",
                            autocomplete: "current-password",
                            required: true,
                            value: "{current}",
                            oninput: move |e: FormEvent| current.set(e.value()),
                        }
                    }
                    Field { label: locale.t("account-new-password"), html_for: "acc-new",
                        Input {
                            id: "acc-new",
                            r#type: "password",
                            autocomplete: "new-password",
                            required: true,
                            minlength: MIN_PASSWORD_LENGTH as i64,
                            maxlength: MAX_PASSWORD_LENGTH as i64,
                            value: "{new_password}",
                            oninput: move |e: FormEvent| new_password.set(e.value()),
                        }
                        PasswordChecklist { id: "acc-new-policy", password: new_password, name, email }
                    }
                    FormError { error: password_error }
                    Button { r#type: "submit", {locale.t("account-change-password")} }
                }
            }
            Section { title: locale.t("account-pending-heading"),
                if data.pending.is_empty() {
                    p { class: "muted", {locale.t("account-pending-empty")} }
                } else {
                    ul { class: "pending-list",
                        for item in data.pending.iter() {
                            li { key: "{item.id}",
                                lucide::Clock { size: 14 }
                                span { {pending_label(locale, item.kind)} ": {item.label}" }
                                if let Some(to) = pending_link(item.kind, item.id) {
                                    Link { to, {locale.t("action-edit")} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn pending_label(locale: crate::i18n::Locale, kind: PendingKind) -> String {
    locale.t(match kind {
        PendingKind::Store => "detail-store",
        PendingKind::Product => "product-form-product",
        PendingKind::Offer => "admin-nav-offers",
        PendingKind::Image => "admin-nav-images",
    })
}

/// Pending stores aren't public yet, so there is no panel to open — but
/// their edit form works (it doesn't filter on approval).
fn pending_link(kind: PendingKind, id: i64) -> Option<Route> {
    match kind {
        PendingKind::Store => Some(Route::EditStore { id }),
        PendingKind::Offer => Some(Route::EditOffer { id }),
        PendingKind::Product | PendingKind::Image => None,
    }
}
