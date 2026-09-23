//! The single `site_info` row (always `id = 1`, enforced by a CHECK).

use sqlx::PgPool;

use crate::models::SiteInfo;

pub async fn get(pool: &PgPool) -> sqlx::Result<SiteInfo> {
    sqlx::query_as!(
        SiteInfo,
        r#"select operator_name, street, postal_code, city, country,
                  email, phone, vat_id, register_number, responsible, purpose
           from site_info where id = 1"#
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &PgPool, info: &SiteInfo, changed_by: i64) -> sqlx::Result<()> {
    sqlx::query!(
        r#"update site_info set
               operator_name = $1, street = $2, postal_code = $3, city = $4,
               country = $5, email = $6, phone = $7, vat_id = $8,
               register_number = $9, responsible = $10, purpose = $11,
               modified_by = $12, modified = now()
           where id = 1"#,
        info.operator_name.trim(),
        info.street.trim(),
        info.postal_code.trim(),
        info.city.trim(),
        info.country.trim(),
        info.email.trim(),
        info.phone.trim(),
        info.vat_id.trim(),
        info.register_number.trim(),
        info.responsible.trim(),
        info.purpose.trim(),
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}
