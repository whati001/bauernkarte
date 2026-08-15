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
mod opening_hours;
mod seasonality;
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
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{cookie::SameSite, session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Must run before `tracing_subscriber::fmt::init()` below: that call
    // reads `RUST_LOG` from the environment right away (via
    // `EnvFilter`'s default-env lookup), and `Config::from_env()` is what
    // actually loads `.env` into the process environment (`dotenvy::dotenv()`,
    // see config.rs). In the previous order, `.env`'s `RUST_LOG` wasn't
    // set yet when the subscriber read it, so it silently fell back to
    // its built-in default filter (ERROR-only) — logs looked like they
    // just weren't happening, even with `RUST_LOG` correctly set in `.env`.
    let config = config::Config::from_env()?;

    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    tracing::info!("database pool connected");

    // tower-sessions-sqlx-store manages its own schema (a `tower_sessions`
    // Postgres schema + `session` table) — no hand-written migration for
    // it (task 2.9's decision).
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    tracing::debug!("session store schema migrated");

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
            patch(handlers::product::update_seasonality).delete(handlers::product::delete),
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
        .route("/api/search/suggest", get(handlers::search::suggest))
        .route("/api/search/select/{kind}/{id}", get(handlers::search::select))
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
        .route("/store-product/{id}/edit", get(handlers::product::edit_seasonality_form))
        .route("/product/{id}/edit", get(handlers::product::edit_product_form))
        .route("/store-product/{id}/image/new", get(handlers::image::new_form))
        .route("/image/{id}", get(handlers::image::show))
        .route("/locale/{code}", get(handlers::locale::switch))
        .merge(rate_limited)
        .nest_service("/static", ServeDir::new("static"))
        .layer(axum::middleware::from_fn(i18n::locale_middleware))
        .layer(session_layer)
        // Outermost layer (applied last = wraps everything else, so it
        // sees every request/response, static assets included) — one
        // `tower_http::trace::TraceLayer` instrumenting *every* route
        // generically beats hand-adding a log line to each of the ~30
        // handlers individually: consistent fields (method, path, status,
        // latency) on every request, and new routes get it for free.
        // Defaults would already do this at DEBUG; bumped to INFO here so
        // `RUST_LOG=product_finder=info` alone (a plausible prod setting)
        // still shows per-request activity, not just this crate's own
        // explicit business-event logs below. Response classification
        // (`ServerErrorsAsFailures`, tower-http's default) treats only
        // 5xx as a failure — this app's 4xx-class rejections (including
        // `AppError::Validation`'s 200-with-form-error, see error.rs's
        // comment on why that's 200) are expected client-driven outcomes,
        // not failures, and are still visible via the INFO-level
        // on_response's `status` field.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
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
