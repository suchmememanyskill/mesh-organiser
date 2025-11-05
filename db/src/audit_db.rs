use crate::{db_context::DbContext, model::{AuditEntry, random_hex_32, time_now}};

pub async fn get_last_audit_entry(db: &DbContext) -> Result<Option<AuditEntry>, sqlx::Error>
{
    let row = sqlx::query!(
        "SELECT audit_unique_global_id, audit_user_id, audit_action_type, audit_entity_type, audit_global_entity_id, audit_created_at
         FROM audit
         ORDER BY audit_created_at DESC
         LIMIT 1"
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => Ok(Some(AuditEntry {
            id: r.audit_unique_global_id,
            user_id: r.audit_user_id,
            action_type: match r.audit_action_type {
                0 => crate::model::ActionType::Create,
                1 => crate::model::ActionType::Update,
                2 => crate::model::ActionType::Delete,
                _ => crate::model::ActionType::Create,
            },
            entity_type: match r.audit_entity_type {
                0 => crate::model::EntityType::Model,
                1 => crate::model::EntityType::Resource,
                2 => crate::model::EntityType::Label,
                3 => crate::model::EntityType::Group,
                _ => crate::model::EntityType::Model,
            },
            global_entity_id: r.audit_global_entity_id,
            timestamp: r.audit_created_at.to_string(),
        })),
        None => Ok(None),
    }
}

pub async fn add_audit_entry(db: &DbContext, entry: &AuditEntry) -> Result<(), sqlx::Error>
{
    /* 
    let last_entry = get_last_audit_entry(db).await?;

    if let Some(last) = last_entry {
        if last.user_id == entry.user_id &&
           last.action_type == entry.action_type &&
           last.entity_type == entry.entity_type {
               // Duplicate entry, skip adding
               return Ok(());
           }
    }*/

    let id = random_hex_32();
    let now = time_now();
    let action_type = entry.action_type as i64;
    let entity_type = entry.entity_type as i64;
    sqlx::query!(
        "INSERT INTO audit (audit_unique_global_id, audit_user_id, audit_action_type, audit_entity_type, audit_global_entity_id, audit_created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
         id,
         entry.user_id,
         action_type,
         entity_type,
         entry.global_entity_id,
         now
    )
    .execute(db)
    .await?;

    Ok(())
}