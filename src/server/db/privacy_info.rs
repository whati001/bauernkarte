//! The single `privacy_info` row (always `id = 1`, enforced by a CHECK).

use sqlx::PgPool;

use crate::models::PrivacyInfo;

pub async fn get(pool: &PgPool) -> sqlx::Result<PrivacyInfo> {
    sqlx::query_as!(
        PrivacyInfo,
        r#"select controller_name, street, postal_code, city, country, email, phone
           from privacy_info where id = 1"#
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &PgPool, info: &PrivacyInfo, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        r#"update privacy_info set
               controller_name = $1, street = $2, postal_code = $3, city = $4,
               country = $5, email = $6, phone = $7,
               modified_by = $8, modified = now()
           where id = 1"#,
        info.controller_name.trim(),
        info.street.trim(),
        info.postal_code.trim(),
        info.city.trim(),
        info.country.trim(),
        info.email.trim(),
        info.phone.trim(),
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}
