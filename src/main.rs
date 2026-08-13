mod auth;
mod config;
mod db;
mod de;
mod dstar;
mod error;
mod handlers;
mod i18n;
mod image_processing;
mod models;
mod sse;
mod state;
mod templates;

use std::{net::SocketAddr, time::Duration as StdDuration};

use axum::{
    routing::{get, patch, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::services::ServeDir;
use tower_sessions::{cookie::SameSite, session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    // tower-sessions-sqlx-store manages its own schema (a `tower_sessions`
    // Postgres schema + `session` table) — no hand-written migration for
    // it (task 2.9's decision).
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    // sqlx-store-backed sessions don't expire rows on their own; run the
    // crate's own cleanup sweep as a background task (task 3.2/design.md's
    // session-cleanup decision) rather than a separate cron process, since
    // v1 is a single-process deployment.
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60 * 60)),
    );

    // design.md's CSRF decision depends specifically on `SameSite=Lax`
    // (blocks the cookie on cross-site POST while still allowing a
    // top-level cross-site GET navigation to arrive logged in) —
    // tower-sessions' own default is `Strict`, which is stronger but
    // not what the no-CSRF-token argument was built on, so it's
    // overridden explicitly rather than left implicit.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.secure_cookies)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(14)));

    let state = AppState { pool };

    // task 3.5 / catalog-editing spec's "rate-limited the same as other
    // mutation routes": login, register, rating, image upload, and every
    // catalog PATCH/DELETE route share this limiter. 8-request burst,
    // refilling one every 500ms per peer IP — `GovernorConfig::default()`'s
    // own preset, made explicit here rather than left implicit.
    let governor_conf = GovernorConfigBuilder::default()
        .per_millisecond(500)
        .burst_size(8)
        .finish()
        .expect("valid governor config");
    let governor_limiter = governor_conf.limiter().clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(StdDuration::from_secs(60));
        loop {
            interval.tick().await;
            governor_limiter.retain_recent();
        }
    });

    let rate_limited = Router::new()
        .route("/login", post(handlers::account::login))
        .route("/register", post(handlers::account::register))
        .route("/store/new", post(handlers::store::create))
        .route("/store/{id}", patch(handlers::store::update).delete(handlers::store::delete))
        .route("/company/{id}", patch(handlers::company::update).delete(handlers::company::delete))
        .route("/store/{id}/product/new", post(handlers::product::create))
        .route(
            "/product/{id}",
            patch(handlers::product::update_product).delete(handlers::product::delete_product),
        )
        .route(
            "/store-product/{id}",
            patch(handlers::product::update).delete(handlers::product::delete),
        )
        .route(
            "/store-product/{id}/rating",
            post(handlers::rating::rate_up).delete(handlers::rating::unrate),
        )
        .route("/store-product/{id}/image", post(handlers::image::upload))
        .route(
            "/image/{id}",
            patch(handlers::image::update).delete(handlers::image::delete),
        )
        .layer(GovernorLayer::new(governor_conf));

    let app = Router::new()
        .route("/healthz", get(handlers::pages::healthz))
        .route("/", get(handlers::pages::index))
        .route("/store/{id}", get(handlers::pages::store_page))
        .route("/api/stores", get(handlers::search::stores))
        .route("/api/filters/categories", get(handlers::search::filter_categories))
        .route("/api/filters/products", get(handlers::search::filter_products))
        .route("/api/store/back", get(handlers::store_detail::back))
        .route("/api/store/{id}", get(handlers::store_detail::show))
        .route("/login", get(handlers::account::login_form))
        .route("/register", get(handlers::account::register_form))
        .route("/logout", post(handlers::account::logout))
        .route("/account", get(handlers::account::account_page).post(handlers::account::update_profile))
        .route("/account/password", post(handlers::account::change_password))
        .route("/store/new", get(handlers::store::new_form))
        .route("/store/{id}/edit", get(handlers::store::edit_form))
        .route("/company/{id}/edit", get(handlers::company::edit_form))
        .route("/store/{id}/product/new", get(handlers::product::new_form))
        .route("/store-product/{id}/edit", get(handlers::product::edit_form))
        .route("/product/{id}/edit", get(handlers::product::edit_product_form))
        .route("/store-product/{id}/image/new", get(handlers::image::new_form))
        .route("/image/{id}", get(handlers::image::show))
        .route("/locale/{code}", get(handlers::locale::switch))
        .merge(rate_limited)
        .nest_service("/static", ServeDir::new("static"))
        .layer(axum::middleware::from_fn(i18n::locale_middleware))
        .layer(session_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    deletion_task.abort();
    Ok(())
}
