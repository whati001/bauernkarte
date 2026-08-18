pub mod account;
pub mod admin;
pub mod company;
pub mod image;
pub mod locale;
pub mod pages;
pub mod product;
pub mod rating;
pub mod search;
pub mod store;
pub mod store_detail;

/// `?store_id=` on a form's own GET route — the store whose detail panel
/// the visitor opened the form from, so the form can offer a way back to
/// it. Not a bound signal: it's baked into the link's `@get(...)` string
/// at render time in `sidebar_detail.html` and rides alongside the
/// `?datastar=` blob Datastar appends to the same URL (see
/// `company::edit_form`, where this pattern started).
#[derive(serde::Deserialize)]
pub struct ReturnQuery {
    #[serde(default)]
    pub store_id: Option<i64>,
}

/// The Datastar action a form's back button fires: the originating
/// store's detail panel when it's known, otherwise the search panel.
///
/// Falling back rather than hiding the button matters — a form reached
/// by a direct URL (or from a link that forgot the query param) still
/// has somewhere to go, instead of stranding the visitor in a panel
/// whose only exit is the browser's own back button.
pub fn back_action(store_id: Option<i64>) -> String {
    match store_id {
        Some(id) => format!("/api/store/{id}"),
        None => "/api/store/back".to_string(),
    }
}
