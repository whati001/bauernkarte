//! Store-detail capability: assembles the full detail view (company +
//! store + product list with ratings/images) from the smaller
//! per-table queries in `db::{store,company,rating,image}`. Kept as its
//! own module since it's a read-side composition, not owned by any single
//! table.

use sqlx::{types::Json, PgPool};

use crate::models::{DayHours, StoreDetail, StoreProductDetail};

struct StoreCompanyRow {
    store_id: i64,
    store_name: String,
    openinghours: Option<Json<Vec<DayHours>>>,
    lat: f64,
    lon: f64,
    company_id: i64,
    company_name: String,
    company_description: Option<String>,
    company_homepage: Option<String>,
}

struct StoreProductRow {
    store_product_id: i64,
    product_id: i64,
    product_name: String,
    product_description: Option<String>,
    product_icon: Option<String>,
    seasonal_months: Option<Json<Vec<i16>>>,
}

pub async fn get_store_detail(
    pool: &PgPool,
    store_id: i64,
    viewer_id: Option<i64>,
) -> sqlx::Result<Option<StoreDetail>> {
    let header = sqlx::query_as!(
        StoreCompanyRow,
        r#"select s.id as "store_id!", s.name as "store_name!",
                  s.openinghours as "openinghours: Json<Vec<DayHours>>",
                  ST_Y(s.position::geometry) as "lat!", ST_X(s.position::geometry) as "lon!",
                  c.id as "company_id!", c.name as "company_name!", c.description as company_description,
                  c.homepage as company_homepage
           from store s
           join company c on c.id = s.company
           where s.id = $1 and s.approved and not s.deleted"#,
        store_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(header) = header else {
        return Ok(None);
    };

    let store_products = sqlx::query_as!(
        StoreProductRow,
        r#"select sp.id as "store_product_id!", p.id as "product_id!", p.name as "product_name!",
                  p.description as product_description, p.icon as product_icon,
                  sp.seasonal_months as "seasonal_months: Json<Vec<i16>>"
           from store_product sp
           join product p on p.id = sp.product and p.approved and not p.deleted
           where sp.store = $1 and sp.approved and not sp.deleted
           order by p.name"#,
        store_id
    )
    .fetch_all(pool)
    .await?;

    let mut products = Vec::with_capacity(store_products.len());
    for row in store_products {
        let ratings = crate::db::rating::counts_for_store_product(pool, row.store_product_id).await?;
        let images = crate::db::image::list_for_store_product(pool, row.store_product_id).await?;
        let viewer_has_rated_up = match viewer_id {
            Some(uid) => crate::db::rating::viewer_has_rated_up(pool, row.store_product_id, uid).await?,
            None => false,
        };
        products.push(StoreProductDetail {
            store_product_id: row.store_product_id,
            product_id: row.product_id,
            product_name: row.product_name,
            product_description: row.product_description,
            product_icon: row.product_icon,
            seasonal_months: row.seasonal_months.map(|j| j.0),
            ratings,
            viewer_has_rated_up,
            images,
        });
    }

    Ok(Some(StoreDetail {
        store_id: header.store_id,
        store_name: header.store_name,
        openinghours: header.openinghours.map(|j| j.0).unwrap_or_default(),
        lat: header.lat,
        lon: header.lon,
        company_id: header.company_id,
        company_name: header.company_name,
        company_description: header.company_description,
        company_homepage: header.company_homepage,
        products,
    }))
}
