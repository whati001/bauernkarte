//! Gives the seeded admin account a password on first startup.
//!
//! The migration that creates `bauernkarte@rehka.dev` can't do this
//! itself: it's static SQL with no access to `.env` and no way to run
//! Argon2, and a hash committed to the repository would be a published
//! password. So the row arrives with an empty `pwd_hash` — which
//! `PasswordHash::new` refuses to parse, making `verify_password` return
//! false — and this fills it in from `ADMIN_PASSWORD`.
//!
//! Runs **once**: only an account whose hash is still empty is touched.
//! That's what lets the admin change their password through /account and
//! keep it across restarts, even with `ADMIN_PASSWORD` still sitting in
//! the environment.

use sqlx::PgPool;

use crate::auth::password;

pub const SEED_ADMIN_EMAIL: &str = "bauernkarte@rehka.dev";

pub async fn apply(pool: &PgPool, admin_password: Option<&str>) -> anyhow::Result<()> {
    let needs_password = sqlx::query_scalar!(
        r#"select exists(
               select 1 from "user" where email = $1 and pwd_hash = ''
           ) as "exists!""#,
        SEED_ADMIN_EMAIL
    )
    .fetch_one(pool)
    .await?;

    if !needs_password {
        return Ok(());
    }

    let Some(admin_password) = admin_password else {
        // Not fatal: the app runs fine, the seeded account just can't be
        // logged into until ADMIN_PASSWORD is provided and it restarts.
        tracing::warn!(
            email = SEED_ADMIN_EMAIL,
            "admin account has no password and ADMIN_PASSWORD is unset — \
             set it in .env and restart to enable the admin login"
        );
        return Ok(());
    };

    // The same policy every other password goes through. A weak
    // ADMIN_PASSWORD should fail loudly at startup rather than quietly
    // create the one account that can moderate everything.
    if let Err(rule) = password::check_policy(admin_password, "BauernKarte Admin", SEED_ADMIN_EMAIL)
    {
        anyhow::bail!(
            "ADMIN_PASSWORD does not meet the password policy ({rule:?}) — \
             see the register form's checklist for the rules"
        );
    }

    let hash = password::hash_password(admin_password)?;
    sqlx::query!(
        r#"update "user" set pwd_hash = $2, modified = now()
           where email = $1 and pwd_hash = ''"#,
        SEED_ADMIN_EMAIL,
        hash
    )
    .execute(pool)
    .await?;

    tracing::info!(email = SEED_ADMIN_EMAIL, "admin password set from ADMIN_PASSWORD");
    Ok(())
}
