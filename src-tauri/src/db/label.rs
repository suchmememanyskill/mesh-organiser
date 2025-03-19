use sqlx;
use sqlx::Row;
use tauri::async_runtime::block_on;

#[derive(sqlx::FromRow)]
pub struct Label {
    #[sqlx(rename = "label_id")]
    pub id: i64,
    #[sqlx(rename = "label_name")]
    pub name: String,
    #[sqlx(rename = "label_color")]
    pub color: u32,
}

#[derive(sqlx::FromRow)]
pub struct LabelExtended {
    pub id: i64,
    pub name: String,
    pub color: i64,
    pub model_count: i64,
}

pub fn get_labels_sync(db: &super::db::Db) -> Vec<LabelExtended> {
    block_on(get_labels(db))
}

pub async fn get_labels(db: &super::db::Db) -> Vec<LabelExtended> {
    let rows = sqlx::query_as!(
		LabelExtended,
		r#"SELECT 
			label_id as id, 
			label_name as name, 
			label_color as color, 
			(SELECT COUNT(*) FROM models_labels WHERE models_labels.label_id = labels.label_id) AS model_count
		  FROM labels"#
	)
	.fetch_all(db)
	.await
	.expect("Failed to get labels");

    return rows;
}

pub fn set_label_on_models_sync(label_id: i64, model_ids: Vec<i64>, db: &super::db::Db) {
    block_on(set_label_on_models(label_id, model_ids, db))
}

pub async fn set_label_on_models(label_id: i64, model_ids: Vec<i64>, db: &super::db::Db) {
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
