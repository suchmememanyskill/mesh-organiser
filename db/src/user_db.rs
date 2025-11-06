use crate::{DbError, db_context::DbContext, model::{User, time_now}};

pub async fn get_users(db: &DbContext) -> Result<Vec<User>, DbError> {
    let rows = sqlx::query_as!(
        User,
        "SELECT user_id AS id, 
            user_name AS username, 
            user_email AS email, 
            user_password_hash AS password_hash,
            user_created_at as created_at, 
            user_last_sync as last_sync, 
            user_sync_token as sync_token,
            user_sync_url as sync_url FROM users"
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}

pub async fn get_user_by_id(db: &DbContext, user_id: i64) -> Result<Option<User>, DbError> {
    let row = sqlx::query_as!(
        User,
        "SELECT user_id AS id, 
            user_name AS username, 
            user_email AS email, 
            user_password_hash AS password_hash,
            user_created_at as created_at, 
            user_last_sync as last_sync, 
            user_sync_token as sync_token,
            user_sync_url as sync_url FROM users WHERE user_id = ?",
        user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(row)
}

pub async fn add_user(db: &DbContext, username: &str, email: &str, password_hash: &str) -> Result<i64, DbError> {
    let now = time_now();

    let result = sqlx::query!(
        "INSERT INTO users (user_name, user_email, user_password_hash, user_created_at) VALUES (?, ?, ?, ?)",
        username,
        email,
        password_hash,
        now
    )
    .execute(db)
    .await?;

    let user_id = result.last_insert_rowid();

    Ok(user_id)
}

pub async fn edit_user(db: &DbContext, user_id: i64, username: &str, email: &str, password_hash: &str, user_last_sync: Option<String>, user_sync_token: Option<String>, user_sync_url: Option<String>) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE users SET user_name = ?, user_email = ?, user_password_hash = ?, user_last_sync = ?, user_sync_token = ?, user_sync_url = ? WHERE user_id = ?",
        username,
        email,
        password_hash,
        user_last_sync,
        user_sync_token,
        user_sync_url,
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_user(db: &DbContext, user_id: i64) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM users WHERE user_id = ?",
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}