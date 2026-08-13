//! Small shared `serde` deserializers for request bodies whose values
//! come from Datastar signals rather than a typed API client.

use serde::{Deserialize, Deserializer};

/// Accepts a JSON number *or* a numeric string and produces `f64`.
///
/// Needed because Datastar's client-side `data-bind` type coercion for a
/// freshly-created signal (one with no prior value in `data-signals`) is
/// not reliably one or the other in practice — writing to two sibling
/// `<input type="number">` elements in the same tick was observed to
/// send one as a JSON number and the other as a JSON string, from
/// otherwise-identical markup (see the store location-picker's git
/// history for the reproduction). Rather than depend on a client-side
/// coercion whose exact rule isn't documented, the server accepts either
/// wire representation.
pub fn flexible_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrFloat {
        String(String),
        Float(f64),
    }
    match StringOrFloat::deserialize(deserializer)? {
        StringOrFloat::Float(f) => Ok(f),
        StringOrFloat::String(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

/// As `flexible_f64`, but `i64` — for the same "not one of Datastar's
/// specially-coerced input types" issue on a `<select>` element (only
/// `number`/`range`/`checkbox` get numeric coercion in the bind plugin;
/// a `<select>`'s value is always sent as a plain string), currently
/// `edit_product_form.html`'s category `<select>`. Sibling `<select>`s
/// elsewhere in the app dodge this by typing the field `String`/
/// `Option<String>` and parsing by hand (see `product.rs`'s
/// `new_product_category_id`) — this one didn't, and so 422'd on every
/// product edit with a generic axum extractor-rejection body that never
/// reaches `AppError::Validation` at all (caught the same way as
/// `return_store_id`: a real submit, not curl).
pub fn flexible_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
    }
    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::Int(i) => Ok(i),
        StringOrInt::String(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

/// As `flexible_f64`, but `Option<i64>` — for `type="hidden"` fields
/// carrying an optional id (currently just `company_form.html`'s
/// `return_store_id`, the same hidden-input-skips-Datastar's-number-
/// coercion issue, caught the same way `store_lat`/`store_lon` were:
/// a real browser round-trip, not curl (the field silently sends a JSON
/// *string*, which `Option<i64>`'s default deserializer rejects outright
/// — every company edit reached via a store's "edit company" link was
/// failing with an invisible 422, since axum's `Json` extractor
/// rejection happens before the handler body — and hence before
/// `AppError::Validation` — ever runs).
pub fn flexible_i64_opt<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Int(i64),
        String(String),
        Null,
    }
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Int(i)) => Ok(Some(i)),
        Some(Value::String(s)) if s.trim().is_empty() => Ok(None),
        Some(Value::String(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
    }
}
