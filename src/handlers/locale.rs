//! Language switcher. A plain server redirect, not a Datastar SSE
//! action: switching language is a whole-page concern (every currently
//! rendered fragment would need re-translating), so a full reload is the
//! simplest correct behavior rather than trying to patch translated text
//! into everything already on screen.

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::i18n::{Locale, LOCALE_COOKIE};

/// `GET /locale/{code}` — sets the locale cookie and redirects back to
/// wherever the user was (via `Referer`), falling back to `/`.
pub async fn switch(Path(code): Path<String>, headers: HeaderMap) -> Response {
    let locale = Locale::from_code(&code);
    // Only a same-origin path, never the raw Referer value as-is — a
    // client-controlled header driving an absolute redirect target is a
    // textbook open-redirect (e.g. a forged `Referer: https://evil.example`
    // sent by something other than a real browser navigation).
    let referer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .and_then(|r| r.parse::<axum::http::Uri>().ok())
        .map(|uri| {
            let mut path = uri.path().to_string();
            if let Some(q) = uri.query() {
                path.push('?');
                path.push_str(q);
            }
            path
        })
        .filter(|p| p.starts_with('/') && !p.starts_with("//"))
        .unwrap_or_else(|| "/".to_string());

    let cookie = format!(
        "{LOCALE_COOKIE}={}; Path=/; Max-Age=31536000; SameSite=Lax",
        locale.code()
    );

    (
        StatusCode::SEE_OTHER,
        [(header::SET_COOKIE, cookie), (header::LOCATION, referer)],
    )
        .into_response()
}
