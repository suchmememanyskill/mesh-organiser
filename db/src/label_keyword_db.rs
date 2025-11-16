use indexmap::IndexMap;

use crate::{DbError, db_context::DbContext, label_db::{self, set_last_updated_on_label}, model::{LabelKeyword, User}, util::time_now};

pub async fn get_keywords_for_label(db: &DbContext, user: &User, label_id: i64) -> Result<Vec<LabelKeyword>, DbError> {
    let rows = sqlx::query!(
        "SELECT keyword_id, keyword_name FROM label_keywords JOIN labels ON label_keywords.keyword_label_id = labels.label_id WHERE keyword_label_id = ? AND label_user_id = ?",
        label_id,
        user.id
    )
    .fetch_all(db)
    .await?;

    let mut result: Vec<LabelKeyword> = Vec::new();
    for row in rows {
        result.push(LabelKeyword {
            id: row.keyword_id.unwrap(),
            name: row.keyword_name,
        });
    }

    Ok(result)
}

pub async fn get_all_keywords(db: &DbContext, user: &User) -> Result<IndexMap<i64, Vec<LabelKeyword>>, DbError> {
    let rows = sqlx::query!(
        "SELECT keyword_id, keyword_name, keyword_label_id FROM label_keywords JOIN labels ON label_keywords.keyword_label_id = labels.label_id WHERE label_user_id = ?",
        user.id
    )
    .fetch_all(db)
    .await?;

    let mut result = IndexMap::new();

    for row in rows {
        let entry = result.entry(row.keyword_label_id).or_insert(Vec::new());
        entry.push(LabelKeyword {
            id: row.keyword_id.unwrap(),
            name: row.keyword_name,
        });
    }

    Ok(result)
}

pub async fn set_keywords_for_label(db: &DbContext, user: &User, label_id: i64, keywords: Vec<String>, update_timestamp : Option<&str>) -> Result<(), DbError> {
    let now = time_now();
    let timestamp = update_timestamp.unwrap_or(&now);
    let hex = label_db::get_unique_id_from_label_id(db, user, label_id).await?;

    sqlx::query!(
        "DELETE FROM label_keywords WHERE keyword_label_id = ?",
        label_id,
    )
    .execute(db)
    .await?;

    for keyword in keywords {
        sqlx::query!(
            "INSERT INTO label_keywords (keyword_name, keyword_label_id) VALUES (?, ?)",
            keyword,
            label_id
        )
        .execute(db)
        .await?;
    }

    set_last_updated_on_label(db, user, label_id, timestamp).await?;

    Ok(())
}