//! The store info panel's read model, composed from the per-table
//! queries: the store, its offers with hearts, photos, the owner portrait
//! and the star rating.

use sqlx::{types::Json, PgPool};

use crate::models::{OfferDetail, StoreDetail};
use crate::server::db::{image, review, store};

pub async fn get_store_detail(pool: &PgPool, store_id: i64, viewer_id: Option<i64>) -> sqlx::Result<Option<StoreDetail>> {
    let Some(s) = store::find_public(pool, store_id).await? else {
        return Ok(None);
    };

    // Hearts and "have I hearted this" in one pass instead of two queries
    // per offer.
    let offers = sqlx::query!(
        r#"select sp.id as "store_product_id!", p.id as "product_id!", p.name as "name!",
                  p.description, p.icon,
                  sp.seasonal_months as "seasonal_months: Json<Vec<i16>>",
                  count(r.id) as "hearts!",
                  coalesce(bool_or(r.created_by = $2), false) as "viewer_has_rated_up!"
           from store_product sp
           join product p on p.id = sp.product and p.approved and not p.deleted
           left join rating r on r.store_product = sp.id
           where sp.store = $1 and sp.approved and not sp.deleted
           group by sp.id, p.id
           order by p.name"#,
        store_id,
        viewer_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| OfferDetail {
        store_product_id: r.store_product_id,
        product_id: r.product_id,
        name: r.name,
        description: r.description,
        icon: r.icon,
        seasonal_months: r.seasonal_months.map(|j| j.0),
        hearts: r.hearts,
        viewer_has_rated_up: r.viewer_has_rated_up,
    })
    .collect();

    Ok(Some(StoreDetail {
        id: s.id,
        name: s.name,
        lat: s.lat,
        lon: s.lon,
        openinghours: s.openinghours.map(|j| j.0).unwrap_or_default(),
        address: s.address,
        phone: s.phone,
        owner_name: s.owner_name,
        owner_since: s.owner_since,
        owner_bio: s.owner_bio,
        owner_image_id: image::owner_portrait(pool, store_id).await?,
        offers,
        photos: image::list_photos(pool, store_id).await?,
        review: review::summary(pool, store_id, viewer_id).await?,
    }))
}
