use sqlx::PgPool;

use crate::models::User;

pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select id, name, email::text as "email!", pwd_hash, verified, created, modified
           from "user" where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select id, name, email::text as "email!", pwd_hash, verified, created, modified
           from "user" where email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await
}

pub async fn email_exists(pool: &PgPool, email: &str) -> sqlx::Result<bool> {
    let row = sqlx::query_scalar!(r#"select exists(select 1 from "user" where email = $1) as "exists!""#, email)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

pub async fn insert(pool: &PgPool, name: &str, email: &str, pwd_hash: &str) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"insert into "user" (name, email, pwd_hash, verified)
           values ($1, $2, $3, false)
           returning id, name, email::text as "email!", pwd_hash, verified, created, modified"#,
        name,
        email,
        pwd_hash
    )
    .fetch_one(pool)
    .await
}

pub async fn update_profile(pool: &PgPool, id: i64, name: &str, email: &str) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"update "user" set name = $2, email = $3, modified = now()
           where id = $1
           returning id, name, email::text as "email!", pwd_hash, verified, created, modified"#,
        id,
        name,
        email
    )
    .fetch_one(pool)
    .await
}

pub async fn update_password(pool: &PgPool, id: i64, pwd_hash: &str) -> sqlx::Result<()> {
    sqlx::query!(
        r#"update "user" set pwd_hash = $2, modified = now() where id = $1"#,
        id,
        pwd_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}
