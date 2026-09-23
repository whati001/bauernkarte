//! "Meine Einträge (in Prüfung)": a user's own not-yet-approved
//! submissions across every moderated table. Four small queries combined
//! here rather than one UNION over differently shaped tables.

use sqlx::PgPool;

use crate::models::{PendingItem, PendingKind};

pub async fn for_user(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<PendingItem>> {
    let mut items = Vec::new();

    for r in sqlx::query!(
        "select id, name from store where created_by = $1 and not approved and not deleted",
        user_id
    )
    .fetch_all(pool)
    .await?
    {
        items.push(PendingItem { kind: PendingKind::Store, id: r.id, label: r.name });
    }

    for r in sqlx::query!(
        "select id, name from product where created_by = $1 and not approved and not deleted",
        user_id
    )
    .fetch_all(pool)
    .await?
    {
        items.push(PendingItem { kind: PendingKind::Product, id: r.id, label: r.name });
    }

    for r in sqlx::query!(
        r#"select sp.id, p.name as product_name, s.name as store_name
           from store_product sp
           join product p on p.id = sp.product
           join store s on s.id = sp.store
           where sp.created_by = $1 and not sp.approved and not sp.deleted"#,
        user_id
    )
    .fetch_all(pool)
    .await?
    {
        items.push(PendingItem {
            kind: PendingKind::Offer,
            id: r.id,
            label: format!("{} @ {}", r.product_name, r.store_name),
        });
    }

    for r in sqlx::query!(
        r#"select i.id, s.name as store_name from image i join store s on s.id = i.store
           where i.created_by = $1 and not i.approved and not i.deleted"#,
        user_id
    )
    .fetch_all(pool)
    .await?
    {
        items.push(PendingItem { kind: PendingKind::Image, id: r.id, label: r.store_name });
    }

    Ok(items)
}
