use crate::{DbError, audit_db, db_context::DbContext, group_db::{self, GroupFilterOptions, GroupOrderBy}, model::{ActionType, AuditEntry, EntityType, ModelGroup, Resource, ResourceFlags, ResourceMeta, User, random_hex_32, time_now}};

pub async fn get_resources(db: &DbContext, user: &User) -> Result<Vec<ResourceMeta>, DbError> {
    let rows = sqlx::query!(
        "SELECT resources.resource_id, resources.resource_name, resources.resource_flags, resources.resource_created, resources.resource_unique_global_id
            FROM resources
            WHERE resources.resource_user_id = ?
            ORDER BY resources.resource_name ASC",
            user.id
    )
    .fetch_all(db)
    .await?;

    let mut resources: Vec<ResourceMeta> = Vec::with_capacity(rows.len());

    for row in rows {
        resources.push(ResourceMeta {
            id: row.resource_id.unwrap(),
            name: row.resource_name,
            flags: ResourceFlags::from_bits(row.resource_flags as u32)
                .unwrap_or(ResourceFlags::empty()),
            created: row.resource_created,
            unique_global_id: row.resource_unique_global_id,
        });
    }

    Ok(resources)
}

pub async fn get_groups_for_resource(db: &DbContext, user: &User, resource_id: i64) -> Result<Vec<ModelGroup>, DbError> {
    let rows = sqlx::query!(
        "SELECT models_group.group_id FROM models_group WHERE models_group.group_resource_id = ? AND models_group.group_user_id = ?",
        resource_id,
        user.id
    )
    .fetch_all(db)
    .await?;

    let groups = group_db::get_groups(db, user, GroupFilterOptions {
        group_ids: Some(rows.iter().map(|r| r.group_id.unwrap()).collect()),
        order_by: Some(GroupOrderBy::NameAsc),
        page: 1,
        page_size: u32::MAX,
        ..Default::default()
    }).await?;

    Ok(groups.items)
}

pub async fn get_resource_meta_by_id(db: &DbContext, user: &User, id: i64) -> Result<Option<ResourceMeta>, DbError> {
    let row = sqlx::query!(
        "SELECT resources.resource_id, resources.resource_name, resources.resource_flags, resources.resource_created, resources.resource_unique_global_id
            FROM resources
            WHERE resources.resource_id = ? AND resources.resource_user_id = ?",
        id,
        user.id
    )
    .fetch_one(db)
    .await;

    match row {
        Ok(row) => Ok(Some(ResourceMeta {
            id: row.resource_id,
            name: row.resource_name,
            flags: ResourceFlags::from_bits(row.resource_flags as u32)
                .unwrap_or(ResourceFlags::empty()),
            created: row.resource_created,
            unique_global_id: row.resource_unique_global_id,
        })),
        Err(DbError::RowNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn add_resource(db: &DbContext, user: &User, name: &str, update_audit : bool) -> Result<i64, DbError> {
    let now = time_now();
    let hex = random_hex_32();

    let result = sqlx::query!(
        "INSERT INTO resources (resource_name, resource_created, resource_user_id, resource_unique_global_id)
            VALUES (?, ?, ?, ?)",
        name,
        now,
        user.id,
        hex
    )
    .execute(db)
    .await?;

    if update_audit {
        audit_db::add_audit_entry(db, &AuditEntry::new(user, ActionType::Create, EntityType::Resource, hex)).await?;
    }

    Ok(result.last_insert_rowid())
}

pub async fn get_unique_id_from_resource_id(db: &DbContext, user: &User, resource_id: i64) -> Result<String, DbError> {
    let row = sqlx::query!(
        "SELECT resource_unique_global_id FROM resources WHERE resource_id = ? AND resource_user_id = ?",
        resource_id,
        user.id
    )
    .fetch_one(db)
    .await?;

    Ok(row.resource_unique_global_id)
}

pub async fn delete_resource(db: &DbContext, user: &User, resource_id: i64, update_audit : bool) -> Result<(), DbError> {
    let hex = get_unique_id_from_resource_id(db, user, resource_id).await?;

    sqlx::query!(
        "DELETE FROM resources WHERE resource_id = ? AND resource_user_id = ?",
        resource_id,
        user.id
    )
    .execute(db)
    .await?;

    if update_audit {
        audit_db::add_audit_entry(db, &AuditEntry::new(user, ActionType::Delete, EntityType::Resource, hex)).await?;
    }

    Ok(())
}

pub async fn edit_resource(db: &DbContext, user: &User, resource_id: i64, name: &str, flags: ResourceFlags, update_audit : bool) -> Result<(), DbError> {
    let bits = flags.bits() as i64;
    let hex = get_unique_id_from_resource_id(db, user, resource_id).await?;

    sqlx::query!(
        "UPDATE resources SET resource_name = ?, resource_flags = ? WHERE resource_id = ? AND resource_user_id = ?",
        name,
        bits,
        resource_id,
        user.id
    )
    .execute(db)
    .await?;

    if update_audit {
        audit_db::add_audit_entry(db, &AuditEntry::new(user, ActionType::Update, EntityType::Resource, hex)).await?;
    }

    Ok(())
}

pub async fn set_resource_on_group(db: &DbContext, user: &User, resource_id: Option<i64>, group_id: i64, update_audit : bool) -> Result<(), DbError> {
    let group = match group_db::get_group_via_id(db, user, group_id).await? {
        Some(g) => g,
        None => {
            return Err(DbError::RowNotFound);
        }
    };

    let hex = match resource_id {
        Some(rid) => Some(get_unique_id_from_resource_id(db, user, rid).await?),
        None => None,
    };

    sqlx::query!(
        "UPDATE models_group SET group_resource_id = ? WHERE group_id = ? AND group_user_id = ?",
        resource_id,
        group_id,
        user.id
    )
    .execute(db)
    .await?;

    if update_audit {
        if let Some(hex) = group.meta.resource_id {
            let hex = get_unique_id_from_resource_id(db, user, hex).await?;
            audit_db::add_audit_entry(db, &AuditEntry::new(user, ActionType::Update, EntityType::Resource, hex)).await?;
        }

        if let Some(hex) = hex {
            audit_db::add_audit_entry(db, &AuditEntry::new(user, ActionType::Update, EntityType::Resource, hex)).await?;
        }
    }

    Ok(())
}