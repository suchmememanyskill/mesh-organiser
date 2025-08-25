
use serde::Serialize;
use sqlx::{self, QueryBuilder};
use tauri::async_runtime::block_on;
use indexmap::IndexMap;

#[derive(Serialize, Debug)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: i64,
    pub children: Vec<LabelMin>,
    pub effective_labels : Vec<LabelMin>,
    pub has_parent : bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct LabelMin {
    pub id: i64,
    pub name: String,
    pub color: i64,
}

// TODO: Allow adding string[] as search parameters to automatically add labels to models

pub fn get_labels_sync(db: &super::db::Db) -> Vec<Label> {
    block_on(get_labels(db))
}

fn get_effective_labels(label_id : i64, effective_labels : &mut Vec<LabelMin>, label_map : &mut IndexMap<i64, Label>) {
    let label = label_map.get(&label_id).unwrap();

    if !effective_labels.iter().any(|l| l.id == label.id) {
        effective_labels.push(LabelMin {
            id: label.id,
            name: label.name.clone(),
            color: label.color,
        });
    }

    let label_child_ids = label.children.iter().map(|l| l.id).collect::<Vec<i64>>();

    label_child_ids.iter().for_each(|f| {
        if !effective_labels.iter().any(|l| l.id == *f) {
            get_effective_labels(f.clone(), effective_labels, label_map);
        }
    });
}

pub async fn get_labels(db: &super::db::Db) -> Vec<Label> {
    let rows = sqlx::query!(
        r#"SELECT 
            parent_labels.label_id  as parent_label_id,
            parent_labels.label_name as parent_label_name,
            parent_labels.label_color as parent_label_color,
            child_labels.label_id as child_label_id, 
            child_labels.label_name as child_label_name, 
            child_labels.label_color as child_label_color
          FROM labels as parent_labels
          LEFT JOIN labels_labels ON parent_labels.label_id = labels_labels.parent_label_id
          LEFT JOIN labels as child_labels ON labels_labels.child_label_id = child_labels.label_id  
          ORDER BY parent_labels.label_name ASC"#
    )
    .fetch_all(db)
    .await
    .expect("Failed to get labels");

    let mut label_map: IndexMap<i64, Label> = IndexMap::new();
    let mut has_parents = vec![];

    for row in rows {
        let entry = label_map.entry(row.parent_label_id).or_insert(Label {
            id: row.parent_label_id,
            name: row.parent_label_name,
            color: row.parent_label_color,
            children: Vec::new(),
            effective_labels: Vec::new(),
            has_parent: false,
        });

        // Hack as silly little sql library doesn't understand that this is optional
        if row.child_label_id <= 0 {
            continue;
        }

        entry.children.push(LabelMin {
            id: row.child_label_id,
            name: row.child_label_name.unwrap(),
            color: row.child_label_color.unwrap(),
        });

        has_parents.push(row.child_label_id);
    }

    for entry in has_parents {
        if let Some(label) = label_map.get_mut(&entry) {
            label.has_parent = true;
        }
    }

    for label_id in label_map.values().map(|l| l.id).collect::<Vec<i64>>() {
        let mut effective_labels = Vec::new();
        get_effective_labels(label_id, &mut effective_labels, &mut label_map);
        let label = label_map.get_mut(&label_id).unwrap();
        label.effective_labels = effective_labels;
    }

    return label_map.into_values().collect();
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

pub fn add_labels_on_model_sync(label_ids: Vec<i64>, model_id: i64, db: &super::db::Db) {
    block_on(add_labels_on_model(label_ids, model_id, db))
}

pub async fn add_labels_on_model(label_ids: Vec<i64>, model_id: i64, db: &super::db::Db) {
    if label_ids.is_empty() {
        return;
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Sqlite> = QueryBuilder::new("INSERT INTO models_labels (label_id, model_id) ");

    query_builder.push_values(label_ids, |mut b, label_id| {
        b.push_bind(label_id).push_bind(model_id);
    });

    let query = query_builder.build();

    query
        .execute(db)
        .await
        .expect("Failed to update labels for models");
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

pub async fn add_childs_to_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    db: &super::db::Db,
) {
    if child_label_ids.is_empty() {
        return;
    }

    let mut query_builder: QueryBuilder<'_, sqlx::Sqlite> = QueryBuilder::new("INSERT INTO labels_labels (parent_label_id, child_label_id) ");

    query_builder.push_values(child_label_ids, |mut b, child_label_id| {
        b.push_bind(parent_label_id).push_bind(child_label_id);
    });

    let query = query_builder.build();

    query
        .execute(db)
        .await
        .expect("Failed to update labels for models");
}

pub async fn remove_childs_from_label(
    parent_label_id: i64, 
    child_label_ids: Vec<i64>, 
    db: &super::db::Db
) {
    let in_query = child_label_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",");

    let formatted_query = format!(
        "DELETE FROM labels_Labels WHERE parent_label_id = ? AND child_label_id IN ({})",
        in_query
    );

    sqlx::query(&formatted_query)
        .bind(parent_label_id)
        .execute(db)
        .await
        .expect("Failed to remove label from models");
}

pub async fn remove_all_childs_from_label(
    parent_label_id: i64, 
    db: &super::db::Db
) {
    sqlx::query!("DELETE FROM labels_labels WHERE parent_label_id = ?", parent_label_id)
        .execute(db)
        .await
        .expect("Failed to remove label from models");
}