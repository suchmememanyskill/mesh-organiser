use crate::{DbError, db_context::DbContext, model::{User, UserPermissions, time_now}};

pub async fn get_users(db: &DbContext) -> Result<Vec<User>, DbError> {
    let rows = sqlx::query!(
        "SELECT user_id AS id, 
            user_name, 
            user_email, 
            user_created_at, 
            user_last_sync, 
            user_sync_token,
            user_sync_url,
            user_permissions FROM users"
    )
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|r| User {
        id: r.id,
        username: r.user_name,
        email: r.user_email,
        created_at: r.user_created_at,
        last_sync: r.user_last_sync,
        sync_token: r.user_sync_token,
        sync_url: r.user_sync_url,
        permissions: UserPermissions::from_bits_truncate(r.user_permissions as u32),
    }).collect())
}

pub async fn get_user_by_id(db: &DbContext, user_id: i64) -> Result<Option<User>, DbError> {
    let row = sqlx::query!(
        "SELECT user_id AS id, 
            user_name, 
            user_email, 
            user_created_at, 
            user_last_sync, 
            user_sync_token,
            user_sync_url,
            user_permissions FROM users WHERE user_id = ?",
        user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| User {
        id: r.id,
        username: r.user_name,
        email: r.user_email,
        created_at: r.user_created_at,
        last_sync: r.user_last_sync,
        sync_token: r.user_sync_token,
        sync_url: r.user_sync_url,
        permissions: UserPermissions::from_bits_truncate(r.user_permissions as u32),
    }))
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

pub async fn edit_user(db: &DbContext, user_id: i64, username: &str, email: &str, user_last_sync: Option<String>, user_sync_token: Option<String>, user_sync_url: Option<String>) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE users SET user_name = ?, user_email = ?, user_last_sync = ?, user_sync_token = ?, user_sync_url = ? WHERE user_id = ?",
        username,
        email,
        user_last_sync,
        user_sync_token,
        user_sync_url,
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn edit_user_password(db: &DbContext, user_id: i64, password_hash: &str) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE users SET user_password_hash = ? WHERE user_id = ?",
        password_hash,
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn set_user_permissions(db: &DbContext, user_id: i64, permissions: UserPermissions) -> Result<(), DbError> {
    let bits = permissions.bits() as i64;
    sqlx::query!(
        "UPDATE users SET user_permissions = ? WHERE user_id = ?",
        bits,
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