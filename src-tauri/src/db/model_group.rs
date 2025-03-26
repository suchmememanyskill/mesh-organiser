use serde::Serialize;
use sqlx;

use tauri::async_runtime::block_on;

#[derive(sqlx::FromRow, Serialize)]
pub struct ModelGroup {
    #[sqlx(rename = "group_id")]
    pub id: i64,
    #[sqlx(rename = "group_name")]
    pub name: String,
    #[sqlx(rename = "group_created")]
    pub created: String,
}

pub fn set_group_id_on_models_sync(group_id: Option<i64>, model_ids: Vec<i64>, db: &super::db::Db) {
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
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query!(
        "INSERT INTO models_group (group_name, group_created) VALUES (?, ?)",
        group_name,
        now
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

pub async fn remove_dead_groups(db: &super::db::Db) {
    sqlx::query!(
        "DELETE FROM models_group
            WHERE group_id NOT IN 
                (SELECT DISTINCT model_group_id 
                    FROM models 
                    WHERE model_group_id IS NOT NULL)"
    )
    .execute(db)
    .await
    .expect("Failed to delete dead groups");
}