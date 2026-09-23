//! Everything that only exists on the server: the Postgres pool, the
//! database layer, sessions and password hashing, image processing, and
//! the handful of plain axum routes that aren't server functions (image
//! bytes, the language switch, the offline page).

pub mod auth;
pub mod db;
pub mod image_processing;
pub mod rate_limit;
pub mod routes;

use std::sync::OnceLock;

use axum::{extract::DefaultBodyLimit, routing::get, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::OnceCell;
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_sessions::{cookie::SameSite, session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::Level;

/// Env-based config, loaded once. Two or three variables don't need a
/// config-file layer.
pub struct Config {
    pub database_url: String,
    /// The `Secure` cookie flag — only meaningful behind real TLS. Set
    /// `SECURE_COOKIES=false` for local HTTP dev, or login silently fails.
    pub secure_cookies: bool,
    /// Applied once, to the seeded admin account, on the first startup
    /// that finds it without a password (see `auth::seed_admin_password`).
    pub admin_password: Option<String>,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        match dotenvy::dotenv() {
            Ok(_) | Err(dotenvy::Error::Io(_)) => {}
            Err(err) => return Err(err.into()),
        }
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            secure_cookies: std::env::var("SECURE_COOKIES").map(|v| v != "false").unwrap_or(true),
            admin_password: std::env::var("ADMIN_PASSWORD").ok().filter(|v| !v.is_empty()),
        })
    }
}

static POOL: OnceLock<PgPool> = OnceLock::new();

/// The shared pool. Set once by `router()` before anything can be served.
pub fn pool() -> &'static PgPool {
    POOL.get().expect("database pool is initialised before serving")
}

/// Pool, migrations, admin seed and the session sweeper — once per
/// process, even though the dev server rebuilds the router on every hot
/// patch.
async fn init_once(config: &Config) -> anyhow::Result<PostgresStore> {
    static STORE: OnceCell<PostgresStore> = OnceCell::const_new();
    STORE
        .get_or_try_init(|| async {
            let pool = PgPoolOptions::new().max_connections(10).connect(&config.database_url).await?;
            tracing::info!("database pool connected");
            sqlx::migrate!("./migrations").run(&pool).await?;
            auth::seed_admin_password(&pool, config.admin_password.as_deref()).await?;

            let store = PostgresStore::new(pool.clone());
            store.migrate().await?;
            // sqlx-backed sessions don't expire rows on their own.
            tokio::task::spawn(
                store.clone().continuously_delete_expired(tokio::time::Duration::from_secs(60 * 60)),
            );
            let _ = POOL.set(pool);
            Ok(store)
        })
        .await
        .cloned()
}

/// The whole app: our plain routes, every server function and the SSR
/// renderer (`dioxus::server::router`), wrapped in sessions, the
/// mutation rate limiter and request tracing.
pub async fn router() -> anyhow::Result<Router> {
    let config = Config::from_env()?;
    let session_store = init_once(&config).await?;

    // `SameSite=Lax` specifically: it blocks the cookie on a cross-site
    // POST (which is what makes CSRF tokens unnecessary here) while still
    // letting a top-level cross-site GET arrive logged in.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.secure_cookies)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(14)));

    Ok(Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/offline", get(routes::offline))
        .route("/locale/{code}", get(routes::switch_locale))
        .route("/image/{id}", get(routes::image))
        .merge(dioxus::server::router(crate::app::App))
        // Uploads are capped at 15 MB in `api::image`; the body limit sits
        // just above that so the handler, not the framework, says why.
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .layer(axum::middleware::from_fn(rate_limit::limit_mutations))
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        ))
}

/// Release builds serve the router themselves so every request carries
/// its peer address (`ConnectInfo`) — the rate limiter keys on it.
/// `dioxus::serve` doesn't attach one, but it's what gives `dx serve` its
/// hot-reloading, so debug builds keep using it (see `main.rs`).
#[cfg(not(debug_assertions))]
pub fn serve_release() -> ! {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async {
        let router = router().await.expect("building the router");
        let addr = dioxus::cli_config::fullstack_address_or_localhost();
        let listener = tokio::net::TcpListener::bind(addr).await.expect("binding the listener");
        tracing::info!("listening on {addr}");
        axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .expect("serving");
    });
    std::process::exit(0)
}
