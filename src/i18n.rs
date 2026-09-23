//! DE/EN translations via Fluent (`locales/*.ftl`, embedded at compile
//! time). Shared by the server and the WASM client: both render the same
//! components, so both need the same strings.
//!
//! The active locale comes from the `locale` cookie (set by
//! `GET /locale/{code}`, see `server::routes`), is read on the server by
//! `api::session::session_info`, and reaches every component as a
//! `Locale` context provided at the app root. Server-rendered HTML and the
//! hydrating client therefore always agree on the language.

use std::collections::HashMap;

use dioxus::prelude::*;
use fluent_templates::{
    fluent_bundle::FluentValue, static_loader, LanguageIdentifier, Loader,
};
use serde::{Deserialize, Serialize};
use unic_langid::langid;

static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "de",
        // No invisible bidi isolation marks around arguments (U+2068 … U+2069):
        // both languages are left-to-right, and the marks end up in
        // copied text and `title` attributes.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub const LOCALE_COOKIE: &str = "locale";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Locale {
    /// The app's original language, and the fallback for a missing or
    /// unrecognised cookie.
    #[default]
    De,
    En,
}

impl Locale {
    fn langid(self) -> LanguageIdentifier {
        match self {
            Locale::De => langid!("de"),
            Locale::En => langid!("en"),
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Locale::De => "de",
            Locale::En => "en",
        }
    }

    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Locale::En,
            _ => Locale::De,
        }
    }

    pub fn t(self, key: &str) -> String {
        LOCALES.lookup(&self.langid(), key)
    }

    /// A message with a single `{ $name }` placeholder.
    pub fn t_name(self, key: &str, name: &str) -> String {
        let mut args = HashMap::new();
        args.insert("name".into(), FluentValue::from(name.to_string()));
        LOCALES.lookup_with_args(&self.langid(), key, &args)
    }

    /// A message with a numeric `{ $count }` that also selects a plural
    /// form. Must go in as a number: Fluent's plural selectors only fire
    /// for numeric values, and a stringly "1" silently falls through to
    /// `*[other]` ("1 Produkte").
    pub fn t_count(self, key: &str, count: i64) -> String {
        let mut args = HashMap::new();
        args.insert("count".into(), FluentValue::from(count));
        LOCALES.lookup_with_args(&self.langid(), key, &args)
    }

    pub fn t_year(self, key: &str, year: i16) -> String {
        let mut args = HashMap::new();
        // A string, not a number: Fluent would group a numeric 2005 as
        // "2.005" in German.
        args.insert("year".into(), FluentValue::from(year.to_string()));
        LOCALES.lookup_with_args(&self.langid(), key, &args)
    }


    /// Server errors travel as translation keys (see `api::error`); anything
    /// that isn't a known key — a transport failure, say — is shown as-is.
    pub fn t_error(self, message: &str) -> String {
        LOCALES
            .try_lookup(&self.langid(), message)
            .unwrap_or_else(|| self.t("error-generic"))
    }
}

/// The locale the app root provided. Cheap to call anywhere in a render.
pub fn use_locale() -> Locale {
    try_consume_context::<Locale>().unwrap_or_default()
}
