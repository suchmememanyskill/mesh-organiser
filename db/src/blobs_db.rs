use crate::{db_context::DbContext, model::Blob};

pub async fn get_blob_via_sha256(db: &DbContext, sha256: &str) -> Result<Option<Blob>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT blob_id, blob_sha256, blob_filetype, blob_size, blob_added FROM blobs WHERE blob_sha256 = ?",
        sha256
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => Ok(Some(Blob {
            id: r.blob_id.unwrap(),
            sha256: r.blob_sha256,
            filetype: r.blob_filetype,
            size: r.blob_size,
            added: r.blob_added.to_string(),
        })),
        None => Ok(None),
    }
}

pub async fn add_blob(db: &DbContext, sha256: &str, filetype: &str, size: i64) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

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

pub async fn delete_dead_blobs(db: &DbContext) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM blobs
            WHERE blob_id NOT IN 
                (SELECT DISTINCT model_blob_id 
                    FROM models 
                    WHERE model_blob_id IS NOT NULL)"
    )
    .execute(db)
    .await?;

    Ok(())
}