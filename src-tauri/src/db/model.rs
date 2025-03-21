use super::label::Label;
use super::model_group::ModelGroup;
use super::{label, model_group};
use serde::Serialize;
use sqlx::Row;
use sqlx::{self, types::chrono};
use std::collections::HashMap;
use tauri::async_runtime::block_on;

#[derive(sqlx::FromRow, Serialize)]
pub struct Model {
    #[sqlx(rename = "model_id")]
    pub id: i64,
    #[sqlx(rename = "model_name")]
    pub name: String,
    #[sqlx(rename = "model_sha256")]
    pub sha256: String,
    #[sqlx(rename = "model_filetype")]
    pub filetype: String,
    #[sqlx(rename = "model_size")]
    pub size: i64,
    #[sqlx(rename = "model_url")]
    pub link: Option<String>,
    #[sqlx(rename = "model_desc")]
    pub description: Option<String>,
    #[sqlx(rename = "model_added")]
    pub added: String,
    #[sqlx(rename = "model_group_id")]
    pub group: Option<ModelGroup>,
    #[sqlx(skip)]
    pub labels: Vec<label::Label>,
}

pub fn get_models_sync(db: &super::db::Db) -> Vec<Model>
{
    block_on(get_models(db))
}

pub async fn get_models(db: &super::db::Db) -> Vec<Model> {
    let rows = sqlx::query!(
        "SELECT models.model_id, model_name, model_sha256, model_filetype, model_url, model_desc, model_group_id, model_added, model_size,
                labels.label_id, label_name, label_color,
                models_group.group_id, group_name
         FROM models 
         LEFT JOIN models_labels ON models.model_id = models_labels.model_id 
         LEFT JOIN labels ON models_labels.label_id = labels.label_id
         LEFT JOIN models_group ON models.model_group_id = models_group.group_id"
    )
    .fetch_all(db)
    .await;

    let mut model_map: HashMap<i64, Model> = HashMap::new();

    for mut row in rows.unwrap() {
        let entry = model_map.entry(row.model_id).or_insert(Model {
            id: row.model_id,
            name: row.model_name,
            sha256: row.model_sha256,
            filetype: row.model_filetype,
            link: row.model_url,
            description: row.model_desc,
            added: row.model_added,
            size: row.model_size,
            group: match row.model_group_id {
                Some(id) => Some(ModelGroup {
                    id: id,
                    name: row.group_name.unwrap(),
                }),
                None => None,
            },
            labels: Vec::new(),
        });

        // Hack as silly little sql library doesn't understand that this is optional
        if row.label_id <= 0
        {
            continue;
        }

        if !entry.labels.iter().any(|f| f.id == (row.label_id)) {
            entry.labels.push(Label {
                id: row.label_id,
                name: row.label_name.take().unwrap(),
                color: row.label_color.take().unwrap(),
            });
        }
    }

    return model_map.into_values().collect();
}

pub fn get_models_by_id_sync(ids: Vec<i64>, db: &super::db::Db) -> Vec<Model>
{
    block_on(get_models_by_id(ids, db))
}

pub async fn get_models_by_id(ids: Vec<i64>, db: &super::db::Db) -> Vec<Model> {
    // Build an IN clause from the ids
    let in_query = ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let formatted_query = format!(
        "SELECT models.model_id, model_name, model_sha256, model_filetype, model_url, model_desc, model_added, model_size,
                labels.label_id, label_name, label_color,
                models_group.group_id, group_name
         FROM models 
         LEFT JOIN models_labels ON models.model_id = models_labels.model_id 
         INNER JOIN labels ON models_labels.label_id = labels.label_id 
         LEFT JOIN models_group ON models.model_group_id = models_group.group_id
         WHERE models.model_id IN ({})", in_query);

    let rows = sqlx::query(&formatted_query).fetch_all(db).await;

    let mut model_map: std::collections::HashMap<i64, Model> = std::collections::HashMap::new();

    for row in rows.unwrap() {
        let model_id: i64 = row.get("model_id");
        let model_name: String = row.get("model_name");
        let model_sha256: String = row.get("model_sha256");
        let model_filetype: String = row.get("model_filetype");
        let model_url: Option<String> = row.get("model_url");
        let model_desc: Option<String> = row.get("model_desc");
        let model_added: String = row.get("model_added");
        let model_size : i64 = row.get("model_size");
        let group_id: Option<i64> = row.get("group_id");
        let group_name: Option<String> = row.get("group_name");
        let mut label_id: Option<i64> = row.get("label_id");
        let mut label_name: Option<String> = row.get("label_name");
        let mut label_color: Option<i64> = row.get("label_color");

        let entry = model_map.entry(model_id).or_insert(Model {
            id: model_id,
            name: model_name,
            sha256: model_sha256,
            filetype: model_filetype,
            link: model_url,
            description: model_desc,
            added: model_added,
            size: model_size,
            group: match group_id {
                Some(id) => Some(ModelGroup {
                    id: id,
                    name: group_name.unwrap(),
                }),
                None => None,
            },
            labels: Vec::new(),
        });

        if label_id.is_none()
        {
            continue;
        }

        let label_id_unwrapped = label_id.take().unwrap();

        if !entry.labels.iter().any(|f| f.id == label_id_unwrapped) {
            entry.labels.push(label::Label {
                id: label_id_unwrapped,
                name: label_name.take().unwrap(),
                color: label_color.take().unwrap(),
            });
        }
    }

    return model_map.into_values().collect();
}

pub fn add_model_sync(name: &str, sha256: &str, filetype: &str, size : i64, db: &super::db::Db) -> i64
{
    block_on(add_model(name, sha256, filetype, size, db))
}

pub async fn add_model(name: &str, sha256: &str, filetype: &str, size : i64, db: &super::db::Db) -> i64 {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query!(
        "INSERT INTO models (model_name, model_sha256, model_added, model_filetype, model_size)
         VALUES (?, ?, ?, ?, ?)",
        name,
        sha256,
        now,
        filetype,
        size,
    )
    .execute(db)
    .await
    .expect("Failed to insert model");

    result.last_insert_rowid()
}

pub fn edit_model_sync(id: i64, name: &str, link: Option<&str>, description: Option<&str>, db: &super::db::Db)
{
    block_on(edit_model(id, name, link, description, db))
}

pub async fn edit_model(id: i64, name: &str, link: Option<&str>, description: Option<&str>, db: &super::db::Db) {
    sqlx::query!(
        "UPDATE models
         SET model_name = ?, model_url = ?, model_desc = ?
         WHERE model_id = ?",
        name,
        link,
        description,
        id
    )
    .execute(db)
    .await
    .expect("Failed to update model");
}

pub fn delete_model_sync(id: i64, db: &super::db::Db)
{
    block_on(delete_model(id, db))
}

pub async fn delete_model(id: i64, db: &super::db::Db) {
    sqlx::query!("DELETE FROM models WHERE model_id = ?", id)
        .execute(db)
        .await
        .expect("Failed to delete model");
}

pub fn get_model_id_via_sha256_sync(sha256: &str, db: &super::db::Db) -> Option<i64>
{
    block_on(get_model_id_via_sha256(sha256, db))
}

pub async fn get_model_id_via_sha256(sha256: &str, db: &super::db::Db) -> Option<i64> {
    let result = sqlx::query!("SELECT model_id FROM models WHERE model_sha256 = ?", sha256)
        .fetch_optional(db)
        .await;

    let mut unwrapped_result = result.unwrap();

    if unwrapped_result.is_some() {
        return Some(unwrapped_result.take().unwrap().model_id);
    }

    return None;
}
