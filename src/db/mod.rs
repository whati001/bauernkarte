pub mod category;
pub mod company;
pub mod detail;
pub mod edit_log;
pub mod image;
pub mod pending;
pub mod product;
pub mod rating;
pub mod store;
pub mod store_product;
pub mod user;

/// Wraps a user-typed term into an `ilike '%…%'` pattern, escaping the
/// wildcards (`%`, `_`) and the escape character itself so the term is
/// matched literally — otherwise typing `%` in the navbar search box
/// would match every row. Shared by the two suggestion queries
/// (`category::search_by_name`, `product::search_approved_by_name`)
/// rather than duplicated in both.
pub fn contains_pattern(term: &str) -> String {
    let escaped = term
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}
