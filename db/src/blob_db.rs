use itertools::join;
use serde::de;

use crate::{DbError, db_context::DbContext, model::Blob, util::time_now};

pub async fn get_blob_via_sha256(db: &DbContext, sha256: &str) -> Result<Option<Blob>, DbError> {
    let row = sqlx::query!(
        "SELECT blob_id, blob_sha256, blob_filetype, blob_size, blob_added FROM blobs WHERE blob_sha256 = ?",
        sha256
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => Ok(Some(Blob {
            id: r.blob_id,
            sha256: r.blob_sha256,
            filetype: r.blob_filetype,
            size: r.blob_size,
            added: r.blob_added.to_string(),
        })),
        None => Ok(None),
    }
}

pub async fn add_blob(db: &DbContext, sha256: &str, filetype: &str, size: i64) -> Result<i64, DbError> {
    let now = time_now();

    let result = sqlx::query!(
        "INSERT INTO blobs (blob_sha256, blob_filetype, blob_size, blob_added) VALUES (?, ?, ?, ?)",
        sha256,
        filetype,
        size,
        now
    )
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn delete_blob(db: &DbContext, blob_id: i64) -> Result<(), DbError> {
    sqlx::query!("DELETE FROM blobs WHERE blob_id = ?", blob_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn get_and_delete_dead_blobs(db: &DbContext) -> Result<Vec<Blob>, DbError> {
    let dead_blob_rows = sqlx::query!(
        "SELECT blob_id, blob_sha256, blob_filetype, blob_size, blob_added FROM blobs
            WHERE blob_id NOT IN 
                (SELECT DISTINCT model_blob_id 
                    FROM models 
                    WHERE model_blob_id IS NOT NULL)"
    )
    .fetch_all(db)
    .await?;

    let mut dead_blobs = Vec::with_capacity(dead_blob_rows.len());

    for row in dead_blob_rows {
        dead_blobs.push(Blob {
            id: row.blob_id,
            sha256: row.blob_sha256,
            filetype: row.blob_filetype,
            size: row.blob_size,
            added: row.blob_added,
        });
    }

    let query = format!(
        "DELETE FROM blobs WHERE blob_id IN ({})",
        join(dead_blobs.iter().map(|r| r.id), ",")
    );

    sqlx::query(&query).execute(db).await?;

    Ok(dead_blobs)
}

pub async fn get_blob_model_usage_count(db : &DbContext, blob_id: i64) -> Result<i64, DbError> {
    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM models WHERE model_blob_id = ?",
        blob_id
    )
    .fetch_one(db)
    .await?;

    Ok(row.count)
}