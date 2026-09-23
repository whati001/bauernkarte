//! All SQL, one module per table/aggregate. Queries are compile-time
//! checked (`sqlx::query!`), against `DATABASE_URL` or the `.sqlx/`
//! offline cache.

pub mod detail;
pub mod edit_log;
pub mod image;
pub mod moderation;
pub mod pending;
pub mod product;
pub mod rating;
pub mod review;
pub mod site_info;
pub mod store;
pub mod store_product;
pub mod user;

