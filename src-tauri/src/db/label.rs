use serde::Serialize;
use sqlx;
use tauri::async_runtime::block_on;

#[derive(sqlx::FromRow, Serialize)]
pub struct Label {
    #[sqlx(rename = "label_id")]
    pub id: i64,
    #[sqlx(rename = "label_name")]
    pub name: String,
    #[sqlx(rename = "label_color")]
    pub color: i64,
}

pub fn get_labels_sync(db: &super::db::Db) -> Vec<Label> {
    block_on(get_labels(db))
}

pub async fn get_labels(db: &super::db::Db) -> Vec<Label> {
    let rows = sqlx::query_as!(
        Label,
        r#"SELECT 
			label_id as id, 
			label_name as name, 
			label_color as color
		  FROM labels"#
    )
    .fetch_all(db)
    .await
    .expect("Failed to get labels");

    return rows;
}

pub fn add_label_on_models_sync(label_id: i64, model_ids: Vec<i64>, db: &super::db::Db) {
    block_on(add_label_on_models(label_id, model_ids, db))
}

pub async fn add_label_on_models(label_id: i64, model_ids: Vec<i64>, db: &super::db::Db) {
    for model_id in model_ids {
        sqlx::query!(
            "INSERT INTO models_labels (label_id, model_id) VALUES (?, ?)",
            label_id,
            model_id
        )
        .execute(db)
        .await
        .expect("Failed to add label to model");
    }
}

pub async fn set_labels_on_model(label_ids: Vec<i64>, model_id: i64, db: &super::db::Db) {
    for label_id in label_ids {
        sqlx::query!(
            "INSERT INTO models_labels (label_id, model_id) VALUES (?, ?)",
            label_id,
            model_id
        )
        .execute(db)
        .await
        .expect("Failed to add label to model");
    }
}

pub async fn remove_labels_from_model(model_id: i64, db: &super::db::Db) {
    sqlx::query!("DELETE FROM models_labels WHERE model_id = ?", model_id)
        .execute(db)
        .await
        .expect("Failed to remove labels from model");
}

pub fn remove_label_from_models_sync(label_id: i64, model_ids: Vec<i64>, db: &super::db::Db) {
    block_on(remove_label_from_models(label_id, model_ids, db))
}

pub async fn remove_label_from_models(label_id: i64, model_ids: Vec<i64>, db: &super::db::Db) {
    let in_query = model_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let formatted_query = format!(
        "DELETE FROM models_labels WHERE label_id = ? AND model_id IN ({})",
        in_query
    );
    sqlx::query(&formatted_query)
        .bind(label_id)
        .execute(db)
        .await
        .expect("Failed to remove label from models");
}

pub fn create_label_sync(name: &str, color: i64, db: &super::db::Db) -> i64 {
    block_on(create_label(name, color, db))
}

pub async fn create_label(name: &str, color: i64, db: &super::db::Db) -> i64 {
    let result = sqlx::query!(
        "INSERT INTO labels (label_name, label_color) VALUES (?, ?)",
        name,
        color
    )
    .execute(db)
    .await
    .expect("Failed to create label");

    result.last_insert_rowid()
}

pub fn edit_label_sync(label_id: i64, name: &str, color: i64, db: &super::db::Db) {
    block_on(edit_label(label_id, name, color, db))
}

pub async fn edit_label(label_id: i64, name: &str, color: i64, db: &super::db::Db) {
    sqlx::query!(
        "UPDATE labels SET label_name = ?, label_color = ? WHERE label_id = ?",
        name,
        color,
        label_id
    )
    .execute(db)
    .await
    .expect("Failed to edit label");
}

pub fn delete_label_sync(label_id: i64, db: &super::db::Db) {
    block_on(delete_label(label_id, db))
}

pub async fn delete_label(label_id: i64, db: &super::db::Db) {
    sqlx::query!("DELETE FROM labels WHERE label_id = ?", label_id)
        .execute(db)
        .await
        .expect("Failed to delete label");
}
