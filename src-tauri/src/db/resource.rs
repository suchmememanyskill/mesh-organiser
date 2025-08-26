use bitflags::bitflags;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::{self, types::chrono};

bitflags! {
    pub struct ResourceFlags: u32 {
        const Completed  = 0b00000001;
    }
}

impl Serialize for ResourceFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut flags = Vec::new();
        if self.contains(ResourceFlags::Completed) {
            flags.push("Completed");
        }
        flags.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ResourceFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let flags: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = ResourceFlags::empty();
        for flag in flags {
            match flag.as_str() {
                "Completed" => result.insert(ResourceFlags::Completed),
                _ => {}
            }
        }
        Ok(result)
    }
}

#[derive(Serialize)]
pub struct Resource {
    pub id: i64,
    pub name: String,
    pub flags: ResourceFlags,
    pub group_ids: Vec<i64>,
    pub created: String,
}

pub async fn get_resources(db: &super::db::Db) -> Vec<Resource> {
    let rows = sqlx::query!(
        "SELECT resources.resource_id, resources.resource_name, resources.resource_flags, resources.resource_created,
            models_group.group_id
            FROM resources
            LEFT JOIN models_group ON resources.resource_id = models_group.group_resource_id
            ORDER BY resources.resource_name ASC"
    )
    .fetch_all(db)
    .await;

    let mut model_map: IndexMap<i64, Resource> = IndexMap::new();

    for row in rows.unwrap() {
        let entry = model_map.entry(row.resource_id).or_insert(Resource {
            id: row.resource_id,
            name: row.resource_name,
            flags: ResourceFlags::from_bits(row.resource_flags as u32)
                .unwrap_or(ResourceFlags::empty()),
            group_ids: match row.group_id {
                Some(id) => vec![id],
                None => Vec::new(),
            },
            created: row.resource_created,
        });

        if row.group_id.is_none() {
            continue;
        }

        let group_id = row.group_id.unwrap();

        if !entry.group_ids.iter().any(|f| f == &group_id) {
            entry.group_ids.push(group_id);
        }
    }

    return model_map.into_values().collect();
}

// TODO: This function does not fetch group ids
pub async fn get_resource_by_id(id: i64, db: &super::db::Db) -> Option<Resource> {
    let row = sqlx::query!(
        "SELECT resources.resource_id, resources.resource_name, resources.resource_flags, resources.resource_created
            FROM resources
            WHERE resources.resource_id = ?",
        id
    )
    .fetch_one(db)
    .await;

    match row {
        Ok(row) => Some(Resource {
            id: row.resource_id,
            name: row.resource_name,
            flags: ResourceFlags::from_bits(row.resource_flags as u32)
                .unwrap_or(ResourceFlags::empty()),
            group_ids: vec![],
            created: row.resource_created,
        }),
        Err(_) => None,
    }
}

pub async fn add_resource(name: &str, db: &super::db::Db) -> i64 {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query!(
        "INSERT INTO resources (resource_name, resource_created) VALUES (?, ?)",
        name,
        now
    )
    .execute(db)
    .await
    .expect("Failed to insert resource");

    result.last_insert_rowid()
}

pub async fn delete_resource(id: i64, db: &super::db::Db) {
    sqlx::query!("DELETE FROM resources WHERE resource_id = ?", id)
        .execute(db)
        .await
        .expect("Failed to delete resource");
}

pub async fn edit_resource(id: i64, name: &str, flags: ResourceFlags, db: &super::db::Db) {
    let bits = flags.bits() as i64;
    sqlx::query!(
        "UPDATE resources SET resource_name = ?, resource_flags = ? WHERE resource_id = ?",
        name,
        bits,
        id
    )
    .execute(db)
    .await
    .expect("Failed to update resource");
}
