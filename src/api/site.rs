use dioxus::prelude::*;

use crate::api::error::ApiResult;
use crate::models::SiteInfo;

#[cfg(feature = "server")]
use crate::server::{db, pool};

/// The Impressum's contents.
#[get("/api/impressum")]
pub async fn impressum() -> ApiResult<SiteInfo> {
    Ok(db::site_info::get(pool()).await?)
}
