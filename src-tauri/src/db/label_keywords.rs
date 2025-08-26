use serde::Serialize;
use tauri::async_runtime::block_on;

#[derive(Serialize, Debug)]
pub struct LabelKeyword {
    pub id: i64,
    pub name: String,
    pub label_id: i64,
}

pub async fn get_keywords_for_label(db: &super::db::Db, label_id: i64) -> Vec<LabelKeyword> {
    let rows = sqlx::query!(
        "SELECT keyword_id, keyword_name FROM label_keywords WHERE keyword_label_id = ?",
        label_id
    )
    .fetch_all(db)
    .await;

    let mut result: Vec<LabelKeyword> = Vec::new();

    if let Ok(rows) = rows {
        for row in rows {
            result.push(LabelKeyword {
                id: row.keyword_id,
                name: row.keyword_name,
                label_id: label_id,
            });
        }
    }

    result
}

pub fn get_all_keywords_sync(db: &super::db::Db) -> Vec<LabelKeyword> {
    block_on(get_all_keywords(db))
}

pub async fn get_all_keywords(db: &super::db::Db) -> Vec<LabelKeyword> {
    let rows =
        sqlx::query!("SELECT keyword_id, keyword_name, keyword_label_id FROM label_keywords")
            .fetch_all(db)
            .await;

    let mut result: Vec<LabelKeyword> = Vec::new();

    if let Ok(rows) = rows {
        for row in rows {
            result.push(LabelKeyword {
                id: row.keyword_id,
                name: row.keyword_name,
                label_id: row.keyword_label_id,
            });
        }
    }

    result
}

pub async fn set_keywords_for_label(db: &super::db::Db, label_id: i64, keywords: Vec<String>) {
    let mut tx = db.begin().await.unwrap();

    sqlx::query!(
        "DELETE FROM label_keywords WHERE keyword_label_id = ?",
        label_id
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    for keyword in keywords {
        let lowercase_keyword = keyword.to_lowercase();
        // TODO: This is a little inefficient, but this interaction won't happen in bulk currently.
        sqlx::query!(
            "INSERT INTO label_keywords (keyword_name, keyword_label_id) VALUES (?, ?)",
            lowercase_keyword,
            label_id
        )
        .execute(&mut *tx)
        .await
        .unwrap();
    }

    tx.commit().await.unwrap();
}
