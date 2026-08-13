//! Env-based config (task 1.2). Loaded once at startup via `dotenvy` +
//! `std::env`; deliberately no config-file layer for v1 — two variables
//! don't need one.

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    /// Bound to the `Secure` cookie flag: only meaningful behind real TLS.
    /// Defaults to `true`; set `SECURE_COOKIES=false` for local HTTP dev
    /// (see design.md's non-functional decisions on the dev/prod split).
    pub secure_cookies: bool,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Missing .env is fine in prod (real env vars set some other way);
        // only a malformed one is worth failing loudly on.
        match dotenvy::dotenv() {
            Ok(_) | Err(dotenvy::Error::Io(_)) => {}
            Err(err) => return Err(err.into()),
        }

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
        let secure_cookies = std::env::var("SECURE_COOKIES")
            .map(|v| v != "false")
            .unwrap_or(true);
        let bind_addr =
            std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        Ok(Self {
            database_url,
            secure_cookies,
            bind_addr,
        })
    }
}
