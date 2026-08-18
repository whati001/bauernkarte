//! The single `site_info` row: operator and contact details behind the
//! Impressum page, editable in the admin area.
//!
//! Always `id = 1` — the table's CHECK constraint guarantees there is
//! never another row, so neither function needs a lookup key.

use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SiteInfo {
    pub operator_name: String,
    pub street: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
    pub email: String,
    pub phone: String,
    pub vat_id: String,
    pub register_number: String,
    pub responsible: String,
    pub purpose: String,
}

impl SiteInfo {
    /// Whether there's enough here to publish. An Impressum with no
    /// operator named isn't a short Impressum, it's an empty one — the
    /// page says so rather than rendering a heading over nothing.
    pub fn is_configured(&self) -> bool {
        !self.operator_name.trim().is_empty()
    }

    /// `postal_code city`, blank if neither is set, so the address block
    /// doesn't render a line containing just a space.
    pub fn postal_city(&self) -> String {
        [self.postal_code.trim(), self.city.trim()]
            .iter()
            .filter(|part| !part.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

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
