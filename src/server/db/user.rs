use sqlx::PgPool;
use time::OffsetDateTime;

use crate::models::SessionUser;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub pwd_hash: String,
    pub admin: bool,
}

impl User {
    pub fn to_session(&self) -> SessionUser {
        SessionUser { id: self.id, name: self.name.clone(), email: self.email.clone(), admin: self.admin }
    }
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select id, name, email::text as "email!", pwd_hash, admin from "user" where id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select id, name, email::text as "email!", pwd_hash, admin from "user" where email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await
}

pub async fn email_exists(pool: &PgPool, email: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar!(r#"select exists(select 1 from "user" where email = $1) as "exists!""#, email)
        .fetch_one(pool)
        .await
}

/// Excluding one account — for a profile update, where keeping your own
/// address isn't a collision.
pub async fn email_taken_by_other(pool: &PgPool, email: &str, id: i64) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from "user" where email = $1 and id <> $2) as "exists!""#,
        email,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn insert(pool: &PgPool, name: &str, email: &str, pwd_hash: &str) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"insert into "user" (name, email, pwd_hash, verified) values ($1, $2, $3, false)
           returning id, name, email::text as "email!", pwd_hash, admin"#,
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
        r#"update "user" set name = $2, email = $3, modified = now() where id = $1
           returning id, name, email::text as "email!", pwd_hash, admin"#,
        id,
        name,
        email
    )
    .fetch_one(pool)
    .await
}

pub async fn update_password(pool: &PgPool, id: i64, pwd_hash: &str) -> sqlx::Result<()> {
    sqlx::query!(r#"update "user" set pwd_hash = $2, modified = now() where id = $1"#, id, pwd_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// A row of the admin user table. `contributions` is what the account
/// leaves behind if deleted (`created_by` is `ON DELETE SET NULL`).
pub struct AdminUserRow {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub admin: bool,
    pub contributions: i64,
    pub created: OffsetDateTime,
}

pub async fn list_all(pool: &PgPool) -> sqlx::Result<Vec<AdminUserRow>> {
    sqlx::query_as!(
        AdminUserRow,
        r#"select u.id, u.name, u.email::text as "email!", u.admin, u.created,
                  (  (select count(*) from store         where created_by = u.id)
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
    sqlx::query!(r#"update "user" set admin = $2, modified = now() where id = $1"#, id, admin)
        .execute(pool)
        .await?;
    Ok(())
}

/// A real delete. Submissions stay in the catalog and lose their author.
pub async fn delete(pool: &PgPool, id: i64) -> sqlx::Result<()> {
    sqlx::query!(r#"delete from "user" where id = $1"#, id).execute(pool).await?;
    Ok(())
}

/// Guards the last-admin case: removing it would lock everyone out of
/// moderation with no way back short of SQL.
pub async fn admin_count(pool: &PgPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"select count(*) as "n!" from "user" where admin"#).fetch_one(pool).await
}
