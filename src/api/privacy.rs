use dioxus::prelude::*;

use crate::models::PrivacyInfo;

#[cfg(feature = "server")]
use crate::server::{auth, db, pool};

/// The controller details used only by the Datenschutz page.
#[get("/api/privacy")]
pub async fn privacy_info() -> crate::api::error::ApiResult<PrivacyInfo> {
    Ok(db::privacy_info::get(pool()).await?)
}

#[get("/api/admin/privacy-info", session: tower_sessions::Session)]
pub async fn admin_privacy_info() -> crate::api::error::ApiResult<PrivacyInfo> {
    auth::require_admin(&session).await?;
    Ok(db::privacy_info::get(pool()).await?)
}

#[post("/api/admin/privacy-info", session: tower_sessions::Session)]
pub async fn admin_save_privacy_info(info: PrivacyInfo) -> crate::api::error::ApiResult<()> {
    let admin = auth::require_admin(&session).await?;
    db::privacy_info::update(pool(), &info, admin.id).await?;
    tracing::info!(admin_id = admin.id, "privacy info updated");
    Ok(())
}
