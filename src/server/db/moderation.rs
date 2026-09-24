//! Queries behind the admin area's moderation queues. One module rather
//! than a per-table split because it's the same four operations over
//! four tables; the parts that genuinely differ (labels, joins) stay as
//! separate compile-time-checked queries.

use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::models::{Entity, FieldDiff, QueueCounts, StoreKind};
use crate::server::db::{self, edit_log::EditAction};

/// The table name, which is also `edit_log.entity_type`.
pub fn table(entity: Entity) -> &'static str {
    match entity {
        Entity::Store => "store",
        Entity::Product => "product",
        Entity::Offer => "store_product",
        Entity::Image => "image",
    }
}

pub struct QueueRow {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub at: OffsetDateTime,
    /// Images only: marked as the store image.
    pub store_image: bool,
}

pub struct ChangeRow {
    pub log_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub at: OffsetDateTime,
    pub diff: Vec<FieldDiff>,
}

pub async fn pending(pool: &PgPool, entity: Entity) -> sqlx::Result<Vec<QueueRow>> {
    match entity {
        Entity::Store => {
            sqlx::query_as!(
                QueueRow,
                r#"select s.id, s.name as "title!", s.address as "subtitle?",
                          u.name as "author?", s.created as "at!",
                          false as "store_image!"
                   from store s left join "user" u on u.id = s.created_by
                   where not s.approved and not s.deleted order by s.created"#
            )
            .fetch_all(pool)
            .await
        }
        Entity::Product => {
            sqlx::query_as!(
                QueueRow,
                r#"select p.id, p.name as "title!", p.description as "subtitle?",
                          u.name as "author?", p.created as "at!",
                          false as "store_image!"
                   from product p left join "user" u on u.id = p.created_by
                   where not p.approved and not p.deleted order by p.created"#
            )
            .fetch_all(pool)
            .await
        }
        Entity::Offer => {
            sqlx::query_as!(
                QueueRow,
                r#"select sp.id, p.name as "title!", s.name as "subtitle?",
                          u.name as "author?", sp.created as "at!",
                          false as "store_image!"
                   from store_product sp
                   join product p on p.id = sp.product
                   join store s on s.id = sp.store
                   left join "user" u on u.id = sp.created_by
                   where not sp.approved and not sp.deleted order by sp.created"#
            )
            .fetch_all(pool)
            .await
        }
        // Title is the shop: a photo has no name, and which shop it claims
        // to show is what an admin is judging.
        Entity::Image => {
            sqlx::query_as!(
                QueueRow,
                r#"select i.id, s.name as "title!", i.description as "subtitle?",
                          u.name as "author?", i.created as "at!",
                          i.cover as "store_image!"
                   from image i
                   join store s on s.id = i.store
                   left join "user" u on u.id = i.created_by
                   where not i.approved and not i.deleted order by i.created"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

/// `modified`/`modified_by`: for a deleted row, when and by whom it was
/// deleted — what an admin deciding on a restore needs.
pub async fn deleted(pool: &PgPool, entity: Entity) -> sqlx::Result<Vec<QueueRow>> {
    match entity {
        Entity::Store => {
            sqlx::query_as!(
                QueueRow,
                r#"select s.id, s.name as "title!", s.address as "subtitle?",
                          u.name as "author?", s.modified as "at!",
                          false as "store_image!"
                   from store s left join "user" u on u.id = s.modified_by
                   where s.deleted order by s.modified desc"#
            )
            .fetch_all(pool)
            .await
        }
        Entity::Product => {
            sqlx::query_as!(
                QueueRow,
                r#"select p.id, p.name as "title!", p.description as "subtitle?",
                          u.name as "author?", p.modified as "at!",
                          false as "store_image!"
                   from product p left join "user" u on u.id = p.modified_by
                   where p.deleted order by p.modified desc"#
            )
            .fetch_all(pool)
            .await
        }
        Entity::Offer => {
            sqlx::query_as!(
                QueueRow,
                r#"select sp.id, p.name as "title!", s.name as "subtitle?",
                          u.name as "author?", sp.modified as "at!",
                          false as "store_image!"
                   from store_product sp
                   join product p on p.id = sp.product
                   join store s on s.id = sp.store
                   left join "user" u on u.id = sp.modified_by
                   where sp.deleted order by sp.modified desc"#
            )
            .fetch_all(pool)
            .await
        }
        Entity::Image => {
            sqlx::query_as!(
                QueueRow,
                r#"select i.id, s.name as "title!", i.description as "subtitle?",
                          u.name as "author?", i.modified as "at!",
                          i.cover as "store_image!"
                   from image i
                   join store s on s.id = i.store
                   left join "user" u on u.id = i.modified_by
                   where i.deleted order by i.modified desc"#
            )
            .fetch_all(pool)
            .await
        }
    }
}

/// Logged edits, newest first, with the changed fields worked out.
pub async fn changes(pool: &PgPool, entity: Entity) -> sqlx::Result<Vec<ChangeRow>> {
    let rows = sqlx::query!(
        r#"select l.id as "log_id!", l.old_value as "old_value!", l.new_value,
                  u.name as "author?", l.changed as "at!"
           from edit_log l left join "user" u on u.id = l.changed_by
           where l.entity_type = $1 and l.action = 'update'
           order by l.changed desc limit 50"#,
        table(entity)
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ChangeRow {
            log_id: r.log_id,
            // The name as it was when edited — which is what the diff is about.
            title: snapshot_title(&r.old_value),
            author: r.author,
            at: r.at,
            diff: diff_snapshots(&r.old_value, r.new_value.as_ref()),
        })
        .collect())
}

fn snapshot_title(snapshot: &Value) -> String {
    snapshot
        .get("name")
        .or_else(|| snapshot.get("description"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("#{}", snapshot.get("id").and_then(Value::as_i64).unwrap_or(0)))
}

/// Changed fields only; `id` and the moderation flags never move in an
/// update and would bury the line that matters.
fn diff_snapshots(old: &Value, new: Option<&Value>) -> Vec<FieldDiff> {
    let (Some(old), Some(new)) = (old.as_object(), new.and_then(Value::as_object)) else {
        return Vec::new();
    };
    old.iter()
        .filter(|(field, _)| !matches!(field.as_str(), "id" | "approved" | "deleted"))
        .filter_map(|(field, old_value)| {
            let new_value = new.get(field)?;
            (old_value != new_value).then(|| FieldDiff {
                field: field.clone(),
                old: render_value(old_value),
                new: render_value(new_value),
            })
        })
        .collect()
}

fn render_value(value: &Value) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

pub async fn counts(pool: &PgPool, entity: Entity) -> sqlx::Result<QueueCounts> {
    // One dynamic statement instead of four identical macro calls; the
    // table name is a `&'static str` from `table`, never request input.
    let sql = format!(
        "select (select count(*) from {t} where not approved and not deleted),
                (select count(*) from {t} where approved and not deleted),
                (select count(*) from edit_log where entity_type = $1 and action = 'update'),
                (select count(*) from {t} where deleted)",
        t = table(entity)
    );
    let (pending, existing, changes, deleted): (Option<i64>, Option<i64>, Option<i64>, Option<i64>) =
        sqlx::query_as(&sql).bind(table(entity)).fetch_one(pool).await?;
    Ok(QueueCounts {
        pending: pending.unwrap_or(0),
        existing: existing.unwrap_or(0),
        changes: changes.unwrap_or(0),
        deleted: deleted.unwrap_or(0),
    })
}

pub enum Outcome {
    Done,
    /// A product restore that would collide with a live row of the same name.
    NameTaken,
}

pub async fn approve(pool: &PgPool, entity: Entity, id: i64, by: i64) -> sqlx::Result<Outcome> {
    set_flag(pool, entity, id, "approved = true", EditAction::Approve, by).await
}

/// A soft delete, so a rejection can be undone like everything else here
/// (and a hard delete would trip the catalog's `NO ACTION` foreign keys).
pub async fn reject(pool: &PgPool, entity: Entity, id: i64, by: i64) -> sqlx::Result<Outcome> {
    set_flag(pool, entity, id, "deleted = true", EditAction::Reject, by).await
}

pub async fn restore(pool: &PgPool, entity: Entity, id: i64, by: i64) -> sqlx::Result<Outcome> {
    if entity == Entity::Product {
        let conflict = sqlx::query_scalar!(
            r#"select live.id from product deleted_row
               join product live on live.name = deleted_row.name
                                and live.id <> deleted_row.id and not live.deleted
               where deleted_row.id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await?;
        if conflict.is_some() {
            return Ok(Outcome::NameTaken);
        }
    }
    set_flag(pool, entity, id, "deleted = false", EditAction::Restore, by).await
}

async fn set_flag(
    pool: &PgPool,
    entity: Entity,
    id: i64,
    assignment: &str,
    action: EditAction,
    by: i64,
) -> sqlx::Result<Outcome> {
    // Both interpolated pieces are fixed strings chosen in this module.
    let sql = format!(
        "update {table} set {assignment}, modified_by = $2, modified = now() where id = $1",
        table = table(entity)
    );
    sqlx::query(&sql).bind(id).bind(by).execute(pool).await?;
    db::edit_log::write(
        pool,
        table(entity),
        id,
        action,
        &serde_json::json!({ "id": id, "action": assignment }),
        None,
        by,
    )
    .await?;
    Ok(Outcome::Done)
}

/// Put a logged edit back by feeding its `old_value` through the entity's
/// own `update`, then log the revert as an ordinary update — so a revert
/// leaves a trace and can itself be reverted.
pub async fn revert(pool: &PgPool, entity: Entity, log_id: i64, by: i64) -> sqlx::Result<()> {
    let entry = sqlx::query!(
        r#"select entity_id as "entity_id!", old_value as "old_value!"
           from edit_log where id = $1 and entity_type = $2 and action = 'update'"#,
        log_id,
        table(entity)
    )
    .fetch_optional(pool)
    .await?;
    let Some(entry) = entry else { return Ok(()) };
    let old = &entry.old_value;
    let id = entry.entity_id;

    let str_field = |key: &str| old.get(key).and_then(Value::as_str).map(str::to_string);
    let json_field = |key: &str| old.get(key).filter(|v| !v.is_null()).cloned();

    let (before, after) = match entity {
        Entity::Store => {
            let Some(before) = db::store::find(pool, id).await? else { return Ok(()) };
            let name = str_field("name").unwrap_or_default();
            let (address, phone) = (str_field("address"), str_field("phone"));
            let (owner_name, owner_bio) = (str_field("owner_name"), str_field("owner_bio"));
            let write = db::store::StoreWrite {
                name: &name,
                kind: StoreKind::from_db(&str_field("kind").unwrap_or(before.kind.clone())),
                lat: old.get("lat").and_then(Value::as_f64).unwrap_or(before.lat),
                lon: old.get("lon").and_then(Value::as_f64).unwrap_or(before.lon),
                openinghours: json_field("openinghours").and_then(|v| serde_json::from_value(v).ok()),
                address: address.as_deref(),
                phone: phone.as_deref(),
                owner_name: owner_name.as_deref(),
                owner_since: old.get("owner_since").and_then(Value::as_i64).map(|y| y as i16),
                owner_bio: owner_bio.as_deref(),
            };
            let after = db::store::update(pool, id, &write, by).await?;
            (db::store::snapshot(&before), db::store::snapshot(&after))
        }
        Entity::Product => {
            let Some(before) = db::product::find(pool, id).await? else { return Ok(()) };
            let name = str_field("name").unwrap_or_default();
            let description = str_field("description");
            // Entries logged before icons were part of the snapshot have
            // no "icon" key: those leave the current icon alone.
            let icon = if old.get("icon").is_some() { str_field("icon") } else { before.icon.clone() };
            let after = db::product::update(pool, id, &name, description.as_deref(), icon.as_deref(), by).await?;
            (db::product::snapshot(&before), db::product::snapshot(&after))
        }
        Entity::Offer => {
            let Some(before) = db::store_product::find(pool, id).await? else { return Ok(()) };
            let months = json_field("seasonal_months").and_then(|v| serde_json::from_value(v).ok());
            let after = db::store_product::update_seasonality(pool, id, months, by).await?;
            (db::store_product::snapshot(&before), db::store_product::snapshot(&after))
        }
        Entity::Image => {
            let Some(before) = db::image::find(pool, id).await? else { return Ok(()) };
            let description = str_field("description");
            // Entries from before the flag existed leave it alone.
            let cover = old.get("cover").and_then(Value::as_bool).unwrap_or(before.cover);
            db::image::update(pool, id, description.as_deref(), cover, by).await?;
            let Some(after) = db::image::find(pool, id).await? else { return Ok(()) };
            (db::image::snapshot(&before), db::image::snapshot(&after))
        }
    };

    db::edit_log::write(pool, table(entity), id, EditAction::Update, &before, Some(&after), by).await
}
