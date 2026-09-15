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
/// reaches `AppError::Validation` at all (caught by a real submit, not curl).
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
