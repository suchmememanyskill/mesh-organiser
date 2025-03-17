use sqlx::{self, types::chrono};
use std::collections::HashMap;
use super::{label, model_group};
use super::label::Label;
use sqlx::Row;

#[derive(sqlx::FromRow)]
pub struct Model {
    #[sqlx(rename = "model_id")]
    pub id : i64,
    #[sqlx(rename = "model_name")]
    pub name : String,
    #[sqlx(rename = "model_sha256")]
    pub sha256 : String,
    #[sqlx(rename = "model_url")]
    pub link : Option<String>,
    #[sqlx(rename = "model_desc")]
    pub description : Option<String>,
    #[sqlx(rename = "model_added")]
    pub added : String,
    #[sqlx(rename = "model_group_id")]
    pub group_id : Option<i64>,
    #[sqlx(skip)]
    pub labels : Vec<label::Label>
}

pub async fn get_models(db : &super::db::Db) -> Vec<Model>
{
    let rows = sqlx::query!(
        "SELECT models.model_id, model_name, model_sha256, model_url, model_desc, model_added, model_group_id, labels.label_id, label_name, label_color 
         FROM models 
         INNER JOIN models_labels ON models.model_id = models_labels.model_id 
         INNER JOIN labels ON models_labels.label_id = labels.label_id"
    )
    .fetch_all(db)
    .await;

    let mut model_map: HashMap<i64, Model> = HashMap::new();

    for row in rows.unwrap()
    {
        let entry = model_map
            .entry(row.model_id)
            .or_insert(Model {
                id: row.model_id,
                name : row.model_name,
                sha256 : row.model_sha256,
                link : row.model_url,
                description : row.model_desc,
                added : row.model_added,
                group_id : row.model_group_id,
                labels : Vec::new()
            });

            if !entry.labels.iter().any(|f| f.id == (row.label_id))
            {
                entry.labels.push(Label {
                    id: row.label_id,
                    name : row.label_name,
                    color : row.label_color as u32
                });
            }
    }

    return model_map.into_values().collect();
}

pub async fn get_models_by_id(ids : Vec<i64>, db : &super::db::Db) -> Vec<Model>
{
    // Build an IN clause from the ids
    let in_query = ids.iter().map(ToString::to_string).collect::<Vec<String>>().join(",");
    let formatted_query = format!(
        "SELECT models.model_id, model_name, model_sha256, model_url, model_desc, model_added, model_group_id, 
                labels.label_id, label_name, label_color 
         FROM models 
         INNER JOIN models_labels ON models.model_id = models_labels.model_id 
         INNER JOIN labels ON models_labels.label_id = labels.label_id 
         WHERE models.model_id IN ({})", in_query);

    let rows = sqlx::query(&formatted_query)
        .fetch_all(db)
        .await;

    let mut model_map: std::collections::HashMap<i64, Model> = std::collections::HashMap::new();

    for row in rows.unwrap() {
        let model_id: i64 = row.get("model_id");
        let model_name: String = row.get("model_name");
        let model_sha256: String = row.get("model_sha256");
        let model_url: Option<String> = row.get("model_url");
        let model_desc: Option<String> = row.get("model_desc");
        let model_added: String = row.get("model_added");
        let model_group_id: Option<i64> = row.get("model_group_id");
        let label_id: i64 = row.get("label_id");
        let label_name: String = row.get("label_name");
        let label_color: i64 = row.get("label_color");

        let entry = model_map.entry(model_id).or_insert(Model {
            id: model_id,
            name: model_name,
            sha256: model_sha256,
            link: model_url,
            description: model_desc,
            added: model_added,
            group_id: model_group_id,
            labels: Vec::new()
        });

        if !entry.labels.iter().any(|f| f.id == label_id) {
            entry.labels.push(label::Label {
                id: label_id,
                name: label_name,
                color: label_color as u32
            });
        }
    }

    return model_map.into_values().collect()
}

pub async fn add_model(name: &str, sha256: &str, db: &super::db::Db) -> i64 
{
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query!(
        "INSERT INTO models (model_name, model_sha256, model_added)
         VALUES (?, ?, ?)",
         name,
         sha256,
         now
    )
    .execute(db)
    .await
    .expect("Failed to insert model");

    result.last_insert_rowid()
}

pub async fn edit_model(id : i64, name: &str, link : &str, description : &str, db : &super::db::Db) 
{
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

pub async fn delete_model(id: i64, db: &super::db::Db) 
{
    sqlx::query!(
        "DELETE FROM models WHERE model_id = ?",
        id
    )
    .execute(db)
    .await
    .expect("Failed to delete model");
}