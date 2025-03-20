use serde::Serialize;
use sqlx;
use sqlx::Row;
use std::collections::HashMap;

use super::{
    label::{self, Label},
    model::Model,
};

use tauri::async_runtime::block_on;

#[derive(sqlx::FromRow, Serialize)]
pub struct ModelGroup {
    #[sqlx(rename = "group_id")]
    pub id: i64,
    #[sqlx(rename = "group_name")]
    pub name: String,
    // TODO: Add creation time
}

/*
pub async fn get_groups_by_id(ids : Vec<u32>, db : &super::db::Db) -> Vec<ModelGroup>
{
    let in_query = ids.iter().map(ToString::to_string).collect::<Vec<String>>().join(",");
    let formatted_query = format!(
        "SELECT models_group.group_id, group_name, labels.label_id, label_name, label_color
         FROM models_group
         INNER JOIN models_group_labels ON models_group.group_id = models_group_labels.group_id
         INNER JOIN labels ON models_group_labels.label_id = labels.label_id
         WHERE models_group.group_id IN ({})", in_query);

    let rows = sqlx::query(&formatted_query)
        .fetch_all(db)
        .await;

    let mut model_group_map: HashMap<i64, ModelGroup> = HashMap::new();

    for row in rows.unwrap() {
        let group_id: i64 = row.get(0);
        let group_name: String = row.get(1);
        let label_id: i64 = row.get(2);
        let label_name: String = row.get(3);
        let label_color: i64 = row.get(4);

        let entry = model_group_map
            .entry(group_id)
            .or_insert(ModelGroup {
                id : group_id,
                name : group_name,
                labels : Vec::new()
            });

        if !entry.labels.iter().any(|f| f.id == (label_id))
        {
            entry.labels.push(Label {
                id: label_id,
                name : label_name,
                color : label_color as u32
            });
        }
    }

    return model_group_map.into_values().collect();
}
*/

pub fn set_group_id_on_models_sync(
	group_id: Option<i64>,
	model_ids: Vec<i64>,
	db: &super::db::Db,
) {
	block_on(set_group_id_on_models(group_id, model_ids, db))
}

pub async fn set_group_id_on_models(
    group_id: Option<i64>,
    model_ids: Vec<i64>,
    db: &super::db::Db,
) {
    let in_query = model_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let formatted_query = format!(
        "UPDATE models
	     SET model_group_id = ?
	     WHERE model_id IN ({})",
        in_query
    );

    sqlx::query(&formatted_query)
        .bind(group_id)
        .execute(db)
        .await
        .unwrap();
}

pub fn add_empty_group_sync(group_name: &str, db: &super::db::Db) -> i64 {
	block_on(add_empty_group(group_name, db))
}

pub async fn add_empty_group(group_name: &str, db: &super::db::Db) -> i64 {
    let result = sqlx::query!(
        "INSERT INTO models_group (group_name) VALUES (?)",
        group_name
    )
    .execute(db)
    .await;

    return result.unwrap().last_insert_rowid();
}


pub fn edit_group_sync(group_id: i64, group_name: &str, db: &super::db::Db) {
	block_on(edit_group(group_id, group_name, db))
}

pub async fn edit_group(group_id: i64, group_name: &str, db: &super::db::Db) {
    sqlx::query!(
        "UPDATE models_group SET group_name = ? WHERE group_id = ?",
        group_name,
        group_id
    )
    .execute(db)
    .await
    .expect("Failed to edit group");
}

pub fn remove_group_sync(group_id: i64, db: &super::db::Db) {
	block_on(remove_group(group_id, db))
}

pub async fn remove_group(group_id: i64, db: &super::db::Db) {
    sqlx::query!("DELETE FROM models_group WHERE group_id = ?", group_id)
        .execute(db)
        .await
        .expect("Failed to delete group");
}
