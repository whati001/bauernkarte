//! content-moderation capability: "Meine Einträge (in Prüfung)" — a
//! user's own not-yet-approved submissions, across every moderated
//! entity type. Five small per-table queries, combined in Rust, rather
//! than one UNION across differently-shaped tables.

use sqlx::PgPool;

pub struct PendingItem {
    pub label: String,
    /// Where "review it" should navigate — `None` for entity types with
    /// no dedicated edit form (currently just `image`, which only has a
    /// description to tweak and isn't worth a special-case link here).
    pub edit_path: Option<String>,
}

impl PendingItem {
    fn new(entity_type: &'static str, id: i64, label: String) -> Self {
        let edit_path = match entity_type {
            "company" => Some(format!("/company/{id}/edit")),
            "store" => Some(format!("/store/{id}/edit")),
            "product" => Some(format!("/product/{id}/edit")),
            "store_product" => Some(format!("/store-product/{id}/edit")),
            _ => None,
        };
        Self { label, edit_path }
    }
}

pub async fn for_user(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<PendingItem>> {
    let mut items = Vec::new();

    let companies = sqlx::query!(
        "select id, name from company where created_by = $1 and not approved and not deleted",
        user_id
    )
    .fetch_all(pool)
    .await?;
    items.extend(
        companies
            .into_iter()
            .map(|r| PendingItem::new("company", r.id, format!("Firma: {}", r.name))),
    );

    let stores = sqlx::query!(
        "select id, name from store where created_by = $1 and not approved and not deleted",
        user_id
    )
    .fetch_all(pool)
    .await?;
    items.extend(
        stores
            .into_iter()
            .map(|r| PendingItem::new("store", r.id, format!("Geschäft: {}", r.name))),
    );

    let products = sqlx::query!(
        "select id, name from product where created_by = $1 and not approved and not deleted",
        user_id
    )
    .fetch_all(pool)
    .await?;
    items.extend(
        products
            .into_iter()
            .map(|r| PendingItem::new("product", r.id, format!("Produkt: {}", r.name))),
    );

    let store_products = sqlx::query!(
        r#"select sp.id, p.name as product_name, s.name as store_name
           from store_product sp
           join product p on p.id = sp.product
           join store s on s.id = sp.store
           where sp.created_by = $1 and not sp.approved and not sp.deleted"#,
        user_id
    )
    .fetch_all(pool)
    .await?;
    items.extend(store_products.into_iter().map(|r| {
        PendingItem::new(
            "store_product",
            r.id,
            format!("Angebot: {} bei {}", r.product_name, r.store_name),
        )
    }));

    let images = sqlx::query!(
        "select id from image where created_by = $1 and not approved and not deleted",
        user_id
    )
    .fetch_all(pool)
    .await?;
    items.extend(
        images
            .into_iter()
            .map(|r| PendingItem::new("image", r.id, format!("Bild #{}", r.id))),
    );

    Ok(items)
}
