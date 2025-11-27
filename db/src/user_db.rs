use rand::Rng;

use crate::{DbError, db_context::DbContext, model::{User, UserPermissions, hash_password}, random_hex_32, time_now};

struct UserDbQuery {
    user_id: i64,
    user_name: String,
    user_email: String,
    user_created_at: String,
    user_last_sync: Option<String>,
    user_sync_token: Option<String>,
    user_sync_url: Option<String>,
    user_permissions: i64,
    user_password_hash: String,
}

impl UserDbQuery {
    fn to_user(self) -> User {
        User {
            id: self.user_id,
            username: self.user_name,
            email: self.user_email,
            created_at: self.user_created_at,
            last_sync: self.user_last_sync,
            sync_token: self.user_sync_token,
            sync_url: self.user_sync_url,
            permissions: UserPermissions::from_bits_truncate(self.user_permissions as u32),
            password_hash: self.user_password_hash,
        }
    }
}

pub async fn get_users(db: &DbContext) -> Result<Vec<User>, DbError> {
    let rows = sqlx::query_as!(
        UserDbQuery,
        "SELECT user_id, 
            user_name, 
            user_email, 
            user_created_at, 
            user_last_sync, 
            user_sync_token,
            user_sync_url,
            user_permissions,
            user_password_hash FROM users"
    )
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|r| r.to_user()).collect())
}

pub async fn get_user_by_id(db: &DbContext, user_id: i64) -> Result<Option<User>, DbError> {
    let row = sqlx::query_as!(
        UserDbQuery,
        "SELECT user_id, 
            user_name, 
            user_email, 
            user_created_at, 
            user_last_sync, 
            user_sync_token,
            user_sync_url,
            user_permissions,
            user_password_hash FROM users WHERE user_id = ?",
        user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| r.to_user()))
}

pub async fn add_user(db: &DbContext, username: &str, email: &str, password: &str) -> Result<i64, DbError> {
    let now = time_now();
    let id = rand::rng().random::<u32>() as i64;
    let password = hash_password(password);

    let result = sqlx::query!(
        "INSERT INTO users (user_id, user_name, user_email, user_password_hash, user_created_at) VALUES (?, ?, ?, ?, ?)",
        id,
        username,
        email,
        password,
        now
    )
    .execute(db)
    .await?;

    let user_id = result.last_insert_rowid();

    Ok(user_id)
}

pub async fn edit_user_min(db: &DbContext, user_id: i64, username: &str, email: &str) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE users SET user_name = ?, user_email = ? WHERE user_id = ?",
        username,
        email,
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
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

pub async fn scramble_validity_token(db: &DbContext, user_id: i64) -> Result<(), DbError> {
    let random = random_hex_32();

    sqlx::query!(
        "UPDATE users SET user_sync_url = ? WHERE user_id = ?",
        random,
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn edit_user_password(db: &DbContext, user_id: i64, password: &str) -> Result<(), DbError> {
    let password = hash_password(password);
    sqlx::query!(
        "UPDATE users SET user_password_hash = ? WHERE user_id = ?",
        password,
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

pub async fn get_user_by_email(db: &DbContext, email: &str) -> Result<Option<User>, DbError> {
    let row = sqlx::query_as!(
        UserDbQuery,
        "SELECT user_id, 
            user_name, 
            user_email, 
            user_created_at, 
            user_last_sync, 
            user_sync_token,
            user_sync_url,
            user_permissions,
            user_password_hash FROM users WHERE user_email = ?",
        email
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| r.to_user()))
}

pub async fn get_user_by_sync_token(db: &DbContext, sync_token: &str) -> Result<Option<User>, DbError> {
    let row = sqlx::query_as!(
        UserDbQuery,
        "SELECT user_id, 
            user_name, 
            user_email, 
            user_created_at, 
            user_last_sync, 
            user_sync_token,
            user_sync_url,
            user_permissions,
            user_password_hash FROM users WHERE user_sync_token = ?",
        sync_token
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| r.to_user()))
}