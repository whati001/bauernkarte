//! The one audit-log writer every catalog edit/delete and every admin
//! action calls. Entity-agnostic: full-row JSON snapshots in and out.

use serde_json::Value;
use sqlx::PgPool;

pub enum EditAction {
    Update,
    Delete,
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
