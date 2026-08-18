//! Task 3.7: the one shared `edit_log` write helper every catalog
//! edit/delete handler calls — see design.md's "Editing & deletion"
//! section. Deliberately entity-agnostic (`entity_type` is a plain
//! string, `old_value`/`new_value` are full-row JSON snapshots), so a
//! single function covers company/store/product/store_product/image
//! instead of one per entity.

use serde_json::Value;
use sqlx::PgPool;

pub enum EditAction {
    Update,
    Delete,
    /// The three below are admin-only, and exist so the moderation UI
    /// leaves a trail of who acted on what — the migration that widened
    /// `edit_log_action_check` is what makes them storable.
    Approve,
    Reject,
    Restore,
}

impl EditAction {
    fn as_str(&self) -> &'static str {
        match self {
            EditAction::Update => "update",
            EditAction::Delete => "delete",
            EditAction::Approve => "approve",
            EditAction::Reject => "reject",
            EditAction::Restore => "restore",
        }
    }
}

pub async fn write(
    pool: &PgPool,
    entity_type: &str,
    entity_id: i64,
    action: EditAction,
    old_value: &Value,
    new_value: Option<&Value>,
    changed_by: i64,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"insert into edit_log (entity_type, entity_id, action, old_value, new_value, changed_by)
           values ($1, $2, $3, $4, $5, $6)"#,
        entity_type,
        entity_id,
        action.as_str(),
        old_value,
        new_value,
        changed_by
    )
    .execute(pool)
    .await?;
    Ok(())
}
