//! The few plain axum routes that aren't server functions: they answer
//! with bytes, a redirect or a standalone document, not with data for a
//! component.

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use tower_sessions::Session;

use crate::{
    i18n::{Locale, LOCALE_COOKIE},
    server::{auth, db, pool},
};

/// The locale from the request's cookie, German when absent.
pub fn locale_from_headers(headers: &HeaderMap) -> Locale {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|cookies| cookies.split(';'))
        .find_map(|kv| {
            let (k, v) = kv.trim().split_once('=')?;
            (k == LOCALE_COOKIE).then(|| Locale::from_code(v))
        })
        .unwrap_or_default()
}

/// `GET /locale/{code}` — sets the cookie and goes back where the visitor
/// was. A full reload on purpose: everything on screen needs retranslating.
/// Only a same-origin path from `Referer` is followed; the raw header as a
/// redirect target would be an open redirect.
pub async fn switch_locale(Path(code): Path<String>, headers: HeaderMap) -> Response {
    let locale = Locale::from_code(&code);
    let back = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .and_then(|r| r.parse::<axum::http::Uri>().ok())
        .map(|uri| match uri.query() {
            Some(q) => format!("{}?{q}", uri.path()),
            None => uri.path().to_string(),
        })
        .filter(|p| p.starts_with('/') && !p.starts_with("//"))
        .unwrap_or_else(|| "/".to_string());
    let cookie = format!("{LOCALE_COOKIE}={}; Path=/; Max-Age=31536000; SameSite=Lax", locale.code());
    (StatusCode::SEE_OTHER, [(header::SET_COOKIE, cookie), (header::LOCATION, back)]).into_response()
}

/// `GET /image/{id}` — the stored bytes, if approved and not deleted, or
/// if the requester uploaded it (so they can preview their pending one).
pub async fn image(Path(id): Path<i64>, session: Session) -> Response {
    let Ok(Some(image)) = db::image::find(pool(), id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let viewer = auth::current_user(&session).await.ok().flatten();
    let is_owner = viewer.is_some_and(|u| Some(u.id) == image.created_by);
    if !(image.approved && !image.deleted) && !is_owner {
        return StatusCode::NOT_FOUND.into_response();
    }
    ([(header::CONTENT_TYPE, image.mime_type)], image.image).into_response()
}

/// `GET /offline` — what `sw.js` serves when a navigation can't reach the
/// network. Its own tiny document: the app shell would go looking for
/// tiles, WASM and server functions that by definition aren't there.
pub async fn offline(headers: HeaderMap) -> Html<String> {
    let l = locale_from_headers(&headers);
    Html(format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
  <title>{title}</title>
  <link rel="icon" type="image/svg+xml" href="/static/favicon.svg">
  <style>
    body {{ font-family: system-ui, sans-serif; display: grid; place-items: center; min-height: 100vh; margin: 0; background: #f9f9f7; color: #0b0b0b; }}
    .offline-card {{ max-width: 28rem; padding: 2rem; text-align: center; }}
    .btn {{ display: inline-block; padding: .5rem 1rem; border-radius: .5rem; background: #008300; color: #fff; text-decoration: none; }}
    @media (prefers-color-scheme: dark) {{ body {{ background: #0d0d0d; color: #fff; }} }}
  </style>
</head>
<body class="offline-page">
  <div class="offline-card">
    <h1>{title}</h1>
    <p>{body}</p>
    <a class="btn" href="/">{retry}</a>
  </div>
</body>
</html>"#,
        lang = l.code(),
        title = l.t("offline-title"),
        body = l.t("offline-body"),
        retry = l.t("offline-retry"),
    ))
}
