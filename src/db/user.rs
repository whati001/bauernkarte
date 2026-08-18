use sqlx::PgPool;

use crate::models::User;

pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select id, name, email::text as "email!", pwd_hash, verified, admin, created, modified
           from "user" where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select id, name, email::text as "email!", pwd_hash, verified, admin, created, modified
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
           returning id, name, email::text as "email!", pwd_hash, verified, admin, created, modified"#,
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
           returning id, name, email::text as "email!", pwd_hash, verified, admin, created, modified"#,
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

/// A row of the admin user table. `contributions` is what the account
/// leaves behind if it's deleted — `created_by` is `ON DELETE SET NULL`
/// everywhere, so the entries survive but lose their author. Showing the
/// number up front is what makes that consequence visible before the
/// click rather than after.
pub struct AdminUserRow {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub admin: bool,
    pub contributions: i64,
    pub created: time::OffsetDateTime,
}

pub async fn list_all(pool: &PgPool) -> sqlx::Result<Vec<AdminUserRow>> {
    sqlx::query_as!(
        AdminUserRow,
        r#"select u.id, u.name, u.email::text as "email!", u.admin, u.created,
                  (  (select count(*) from company       where created_by = u.id)
                   + (select count(*) from store         where created_by = u.id)
                   + (select count(*) from product       where created_by = u.id)
                   + (select count(*) from store_product where created_by = u.id)
                   + (select count(*) from image         where created_by = u.id)
                  ) as "contributions!"
           from "user" u
           order by u.admin desc, lower(u.name)"#
    )
    .fetch_all(pool)
    .await
}

pub async fn set_admin(pool: &PgPool, id: i64, admin: bool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"update "user" set admin = $2, modified = now() where id = $1"#,
        id,
        admin
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: i64) -> sqlx::Result<()> {
    // A real delete, not a flag. Every `created_by`/`modified_by` is
    // `ON DELETE SET NULL`, so the account's submissions stay in the
    // catalog and only lose their attribution. Unlike everything else in
    // the admin UI this cannot be undone — the handler confirms first.
    sqlx::query!(r#"delete from "user" where id = $1"#, id)
        .execute(pool)
        .await?;
    Ok(())
}

/// How many admins are left. Guards the last-admin case: removing the
/// final admin (by demotion or deletion) would lock everyone out of the
/// moderation UI with no way back in short of SQL.
pub async fn admin_count(pool: &PgPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"select count(*) as "n!" from "user" where admin"#)
        .fetch_one(pool)
        .await
}
