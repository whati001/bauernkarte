//! Queries behind the admin UI — the moderation queues that replace the
//! hand-run SQL in `RUNBOOK.md`.
//!
//! One module rather than a per-table split (the house rule elsewhere in
//! `db/`) because everything here is the *same* four operations applied
//! to five tables; splitting it would put five near-identical copies of
//! each into five files. The differences that are real — which columns
//! make a readable label, which joins reach them — stay as separate
//! compile-time-checked queries below.

use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::db::edit_log::EditAction;

/// The five moderated tables. `category` is deliberately absent: it's a
/// fixed taxonomy managed directly in the database, not user-creatable
/// (see the comment on the table in its migration).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entity {
    Company,
    Store,
    Product,
    /// `store_product` — "this shop sells this product", with its season.
    Offer,
    Image,
}

impl Entity {
    pub const ALL: [Entity; 5] = [
        Entity::Company,
        Entity::Store,
        Entity::Product,
        Entity::Offer,
        Entity::Image,
    ];

    /// URL segment under `/admin/`.
    pub fn slug(self) -> &'static str {
        match self {
            Entity::Company => "companies",
            Entity::Store => "stores",
            Entity::Product => "products",
            Entity::Offer => "offers",
            Entity::Image => "images",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.slug() == slug)
    }

    /// The value `edit_log.entity_type` uses, which is also the table
    /// name — the two have always been the same string.
    pub fn table(self) -> &'static str {
        match self {
            Entity::Company => "company",
            Entity::Store => "store",
            Entity::Product => "product",
            Entity::Offer => "store_product",
            Entity::Image => "image",
        }
    }

    /// Fluent key for the section's own name.
    pub fn label_key(self) -> &'static str {
        match self {
            Entity::Company => "admin-nav-companies",
            Entity::Store => "admin-nav-stores",
            Entity::Product => "admin-nav-products",
            Entity::Offer => "admin-nav-offers",
            Entity::Image => "admin-nav-images",
        }
    }
}

/// One line in a queue. Deliberately flat and pre-rendered: the template
/// shows a label, a bit of context and who to blame, and every entity
/// reaches those through different joins.
pub struct QueueRow {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub at: OffsetDateTime,
}

pub struct Counts {
    pub pending: i64,
    pub changes: i64,
    pub deleted: i64,
}

/// A logged edit, with the fields that actually differ already worked out.
pub struct ChangeRow {
    pub log_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub at: OffsetDateTime,
    pub diff: Vec<FieldDiff>,
}

pub struct FieldDiff {
    pub field: String,
    pub old: String,
    pub new: String,
}

// ---------------------------------------------------------------------
// Queues
// ---------------------------------------------------------------------

pub async fn pending(pool: &PgPool, entity: Entity) -> sqlx::Result<Vec<QueueRow>> {
    let rows = match entity {
        Entity::Company => sqlx::query_as!(
            QueueRow,
            r#"select c.id, c.name as "title!", null as "subtitle?",
                      u.name as "author?", c.created as "at!"
               from company c left join "user" u on u.id = c.created_by
               where not c.approved and not c.deleted order by c.created"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Store => sqlx::query_as!(
            QueueRow,
            r#"select s.id, s.name as "title!", co.name as "subtitle?",
                      u.name as "author?", s.created as "at!"
               from store s
               join company co on co.id = s.company
               left join "user" u on u.id = s.created_by
               where not s.approved and not s.deleted order by s.created"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Product => sqlx::query_as!(
            QueueRow,
            r#"select p.id, p.name as "title!", cat.name as "subtitle?",
                      u.name as "author?", p.created as "at!"
               from product p
               join category cat on cat.id = p.category
               left join "user" u on u.id = p.created_by
               where not p.approved and not p.deleted order by p.created"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Offer => sqlx::query_as!(
            QueueRow,
            r#"select sp.id, p.name as "title!", s.name as "subtitle?",
                      u.name as "author?", sp.created as "at!"
               from store_product sp
               join product p on p.id = sp.product
               join store s on s.id = sp.store
               left join "user" u on u.id = sp.created_by
               where not sp.approved and not sp.deleted order by sp.created"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Image => sqlx::query_as!(
            QueueRow,
            r#"select i.id, coalesce(i.description, p.name) as "title!",
                      s.name as "subtitle?", u.name as "author?", i.created as "at!"
               from image i
               join store_product sp on sp.id = i.store_product
               join product p on p.id = sp.product
               join store s on s.id = sp.store
               left join "user" u on u.id = i.created_by
               where not i.approved and not i.deleted order by i.created"#
        )
        .fetch_all(pool)
        .await?,
    };
    Ok(rows)
}

pub async fn deleted(pool: &PgPool, entity: Entity) -> sqlx::Result<Vec<QueueRow>> {
    // `modified`/`modified_by` rather than `created`: for a deleted row
    // those are when and by whom it was deleted (the soft-delete writes
    // both), which is what an admin deciding whether to restore needs.
    let rows = match entity {
        Entity::Company => sqlx::query_as!(
            QueueRow,
            r#"select c.id, c.name as "title!", null as "subtitle?",
                      u.name as "author?", c.modified as "at!"
               from company c left join "user" u on u.id = c.modified_by
               where c.deleted order by c.modified desc"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Store => sqlx::query_as!(
            QueueRow,
            r#"select s.id, s.name as "title!", co.name as "subtitle?",
                      u.name as "author?", s.modified as "at!"
               from store s
               join company co on co.id = s.company
               left join "user" u on u.id = s.modified_by
               where s.deleted order by s.modified desc"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Product => sqlx::query_as!(
            QueueRow,
            r#"select p.id, p.name as "title!", cat.name as "subtitle?",
                      u.name as "author?", p.modified as "at!"
               from product p
               join category cat on cat.id = p.category
               left join "user" u on u.id = p.modified_by
               where p.deleted order by p.modified desc"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Offer => sqlx::query_as!(
            QueueRow,
            r#"select sp.id, p.name as "title!", s.name as "subtitle?",
                      u.name as "author?", sp.modified as "at!"
               from store_product sp
               join product p on p.id = sp.product
               join store s on s.id = sp.store
               left join "user" u on u.id = sp.modified_by
               where sp.deleted order by sp.modified desc"#
        )
        .fetch_all(pool)
        .await?,
        Entity::Image => sqlx::query_as!(
            QueueRow,
            r#"select i.id, coalesce(i.description, p.name) as "title!",
                      s.name as "subtitle?", u.name as "author?", i.modified as "at!"
               from image i
               join store_product sp on sp.id = i.store_product
               join product p on p.id = sp.product
               join store s on s.id = sp.store
               left join "user" u on u.id = i.modified_by
               where i.deleted order by i.modified desc"#
        )
        .fetch_all(pool)
        .await?,
    };
    Ok(rows)
}

/// Logged edits, newest first, with the changed fields worked out from
/// the two JSON snapshots. `edit_log` is one polymorphic table, so unlike
/// the queues above this needs no per-entity branch.
pub async fn changes(pool: &PgPool, entity: Entity) -> sqlx::Result<Vec<ChangeRow>> {
    let rows = sqlx::query!(
        r#"select l.id as "log_id!", l.old_value as "old_value!", l.new_value,
                  u.name as "author?", l.changed as "at!"
           from edit_log l left join "user" u on u.id = l.changed_by
           where l.entity_type = $1 and l.action = 'update'
           order by l.changed desc limit 50"#,
        entity.table()
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let diff = diff_snapshots(&r.old_value, r.new_value.as_ref());
            ChangeRow {
                log_id: r.log_id,
                // The snapshot's own name, not the row's current one: it
                // says what the entry was called when it was edited,
                // which is what the diff below is about.
                title: snapshot_title(&r.old_value),
                author: r.author,
                at: r.at,
                diff,
            }
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

/// Fields whose value changed. `id` and the moderation flags are skipped:
/// they never move in an `update`, and showing "approved: false ->
/// false" on every row buries the one line that matters.
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

pub async fn counts(pool: &PgPool, entity: Entity) -> sqlx::Result<Counts> {
    // One dynamic statement instead of five identical macro invocations
    // that differ only in a table name. The name is a `&'static str` off
    // the enum above — never anything a request supplies — so there is no
    // interpolation risk, and the shape is fixed enough that compile-time
    // checking would only be restating it.
    let sql = format!(
        "select (select count(*) from {table} where not approved and not deleted) as pending,
                (select count(*) from edit_log where entity_type = $1 and action = 'update') as changes,
                (select count(*) from {table} where deleted) as deleted",
        table = entity.table()
    );
    let row: (Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(&sql)
        .bind(entity.table())
        .fetch_one(pool)
        .await?;
    Ok(Counts {
        pending: row.0.unwrap_or(0),
        changes: row.1.unwrap_or(0),
        deleted: row.2.unwrap_or(0),
    })
}

// ---------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------

/// What a moderation action produced, so the handler can say so.
pub enum Outcome {
    Done,
    /// A restore that would collide with a live row — currently only
    /// reachable for `product`, whose name is unique among non-deleted
    /// rows. Reported rather than surfaced as a constraint violation.
    NameTaken(String),
}

pub async fn approve(pool: &PgPool, entity: Entity, id: i64, by: i64) -> sqlx::Result<Outcome> {
    set_flag(pool, entity, id, "approved = true", EditAction::Approve, by).await
}

/// Rejecting is a soft delete, not the `delete from …` the runbook used.
/// Two reasons: the catalog's foreign keys are all `NO ACTION`, so a hard
/// delete of a store that already has an offer fails outright; and a
/// rejected row that still exists can be un-rejected, which makes every
/// action in this UI reversible. Purging for real stays a deliberate SQL
/// job outside the app.
pub async fn reject(pool: &PgPool, entity: Entity, id: i64, by: i64) -> sqlx::Result<Outcome> {
    set_flag(pool, entity, id, "deleted = true", EditAction::Reject, by).await
}

pub async fn restore(pool: &PgPool, entity: Entity, id: i64, by: i64) -> sqlx::Result<Outcome> {
    // `product_name_key` only covers non-deleted rows (so a deleted name
    // can be reused). The flip side is that restoring can collide, and a
    // raw unique-violation would reach the visitor as a 500.
    if entity == Entity::Product
        && let Some(name) = product_name_conflict(pool, id).await?
    {
        return Ok(Outcome::NameTaken(name));
    }
    set_flag(pool, entity, id, "deleted = false", EditAction::Restore, by).await
}

async fn product_name_conflict(pool: &PgPool, id: i64) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar!(
        r#"select live.name from product deleted_row
           join product live on live.name = deleted_row.name
                            and live.id <> deleted_row.id and not live.deleted
           where deleted_row.id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

async fn set_flag(
    pool: &PgPool,
    entity: Entity,
    id: i64,
    assignment: &str,
    action: EditAction,
    by: i64,
) -> sqlx::Result<Outcome> {
    // Same reasoning as `counts`: `assignment` and the table name are
    // both fixed strings chosen in this module, never request input.
    let sql = format!(
        "update {table} set {assignment}, modified_by = $2, modified = now() where id = $1",
        table = entity.table()
    );
    sqlx::query(&sql).bind(id).bind(by).execute(pool).await?;

    // Who approved what — the reason the edit_log CHECK was widened.
    // `old_value` carries the identifying pair rather than a full row
    // snapshot: the row itself is untouched apart from one flag, so a
    // copy of it would say nothing the entry doesn't already.
    crate::db::edit_log::write(
        pool,
        entity.table(),
        id,
        action,
        &serde_json::json!({ "id": id, "action": assignment }),
        None,
        by,
    )
    .await?;
    Ok(Outcome::Done)
}

/// Put a logged edit back: read `old_value` from `edit_log` and feed it
/// through the entity's own `update`, which re-validates the row and
/// bumps `modified_by` exactly as a human edit would.
///
/// This is why the snapshots in each `db::*::snapshot` are shaped like
/// their `update` arguments — the runbook called reverting "a manual,
/// one-row-at-a-time operation", and it stayed manual only because
/// nothing walked that mapping.
///
/// The revert is itself logged as an ordinary `update`, taken from the
/// row as it stands right now. Two reasons: without it a revert would be
/// the one change to the catalog that leaves no trace, and with it a
/// revert can be reverted — there is no dead end, and the "changes" tab
/// tells the whole story in order. (`db::*::update` doesn't write the
/// log itself; every caller does, so this one does too.)
pub async fn revert(pool: &PgPool, entity: Entity, log_id: i64, by: i64) -> sqlx::Result<()> {
    let entry = sqlx::query!(
        r#"select entity_id as "entity_id!", old_value as "old_value!"
           from edit_log where id = $1 and entity_type = $2 and action = 'update'"#,
        log_id,
        entity.table()
    )
    .fetch_optional(pool)
    .await?;

    let Some(entry) = entry else {
        return Ok(());
    };
    let old = &entry.old_value;
    let id = entry.entity_id;

    let str_field = |key: &str| old.get(key).and_then(Value::as_str).map(str::to_string);
    let i64_field = |key: &str| old.get(key).and_then(Value::as_i64);
    let f64_field = |key: &str| old.get(key).and_then(Value::as_f64);

    // Snapshot of where the row stands *before* the revert, so the log
    // entry reads like any other edit: this -> that.
    let (before, after) = match entity {
        Entity::Company => {
            let before = crate::db::company::find(pool, id).await?;
            let Some(before) = before else { return Ok(()) };
            let after = crate::db::company::update(
                pool,
                id,
                &str_field("name").unwrap_or_default(),
                str_field("description").as_deref(),
                str_field("homepage").as_deref(),
                by,
            )
            .await?;
            (
                crate::db::company::snapshot(&before),
                crate::db::company::snapshot(&after),
            )
        }
        Entity::Store => {
            let before = crate::db::store::find(pool, id).await?;
            let Some(before) = before else { return Ok(()) };
            let hours = old
                .get("openinghours")
                .filter(|v| !v.is_null())
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let after = crate::db::store::update(
                pool,
                id,
                i64_field("company").unwrap_or(before.company),
                &str_field("name").unwrap_or_default(),
                f64_field("lat").unwrap_or(before.lat),
                f64_field("lon").unwrap_or(before.lon),
                hours,
                by,
            )
            .await?;
            (
                crate::db::store::snapshot(&before),
                crate::db::store::snapshot(&after),
            )
        }
        Entity::Product => {
            let before = crate::db::product::find(pool, id).await?;
            let Some(before) = before else { return Ok(()) };
            let after = crate::db::product::update(
                pool,
                id,
                i64_field("category").unwrap_or(before.category),
                &str_field("name").unwrap_or_default(),
                str_field("description").as_deref(),
                by,
            )
            .await?;
            (
                crate::db::product::snapshot(&before),
                crate::db::product::snapshot(&after),
            )
        }
        Entity::Offer => {
            let before = crate::db::store_product::find(pool, id).await?;
            let Some(before) = before else { return Ok(()) };
            let months = old
                .get("seasonal_months")
                .filter(|v| !v.is_null())
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let after = crate::db::store_product::update_seasonality(pool, id, months, by).await?;
            (
                crate::db::store_product::snapshot(&before),
                crate::db::store_product::snapshot(&after),
            )
        }
        Entity::Image => {
            let before = crate::db::image::find(pool, id).await?;
            let Some(before) = before else { return Ok(()) };
            let after = crate::db::image::update_description(
                pool,
                id,
                str_field("description").as_deref(),
                by,
            )
            .await?;
            (
                crate::db::image::snapshot(&before),
                crate::db::image::snapshot(&after),
            )
        }
    };

    crate::db::edit_log::write(
        pool,
        entity.table(),
        id,
        EditAction::Update,
        &before,
        Some(&after),
        by,
    )
    .await?;
    Ok(())
}
