use sqlx::QueryBuilder;

use crate::{DbError, db_context::DbContext, model::{Share, User}, model_db, random_hex_32, time_now};

pub async fn get_shares(
    db: &DbContext,
    user: &User,
) -> Result<Vec<Share>, DbError> {
    let shares = sqlx::query!("
        SELECT shares.share_id, share_user_id, share_created_at, share_name, GROUP_CONCAT(shares_models.model_id) AS \"share_model_ids: String\"
        FROM shares
        LEFT JOIN shares_models ON shares.share_id = shares_models.share_id
        WHERE share_user_id = ?
        GROUP BY shares.share_id
        ORDER BY share_created_at DESC
        ", user.id)
        .fetch_all(db)
        .await?;

    Ok(shares.into_iter().map(|share| {
        let model_ids: Vec<i64> = match share.share_model_ids {
            Some(ids_str) => ids_str
                .split(',')
                .filter_map(|s| s.parse::<i64>().ok())
                .collect(),
            None => Vec::new(),
        };

        Share {
            id: share.share_id,
            created_at: share.share_created_at,
            share_name: share.share_name,
            user_id: share.share_user_id,
            model_ids,
        }
    }).collect())
}

pub async fn get_share_via_id(
    db: &DbContext,
    share_id: &str,
) -> Result<Share, DbError> {
    let share = sqlx::query!("
        SELECT shares.share_id, share_user_id, share_created_at, share_name, GROUP_CONCAT(shares_models.model_id) AS \"share_model_ids: String\"
        FROM shares
        LEFT JOIN shares_models ON shares.share_id = shares_models.share_id
        WHERE shares.share_id = ?
        GROUP BY shares.share_id
        ", share_id)
        .fetch_one(db)
        .await?;

    let model_ids: Vec<i64> = match share.share_model_ids {
        Some(ids_str) => ids_str
            .split(',')
            .filter_map(|s| s.parse::<i64>().ok())
            .collect(),
        None => Vec::new(),
    };

    Ok(Share {
        id: share.share_id,
        created_at: share.share_created_at,
        share_name: share.share_name,
        user_id: share.share_user_id,
        model_ids: model_ids,
    })
}

pub async fn set_model_ids_on_share(
    db: &DbContext,
    user: &User,
    share_id: &str,
    model_ids: Vec<i64>,
) -> Result<(), DbError> {
    let models = model_db::get_models_via_ids(db, user, model_ids.clone()).await?;

    if models.len() != model_ids.len() {
        return Err(DbError::RowNotFound);
    }

    let share = sqlx::query!("SELECT share_id FROM shares WHERE share_id = ? AND share_user_id = ?", share_id, user.id)
        .fetch_optional(db)
        .await?;

    if share.is_none() {
        return Err(DbError::RowNotFound);
    }

    sqlx::query!("DELETE FROM shares_models WHERE share_id = ?", share_id)
        .execute(db)
        .await?;

    let mut query_builder = QueryBuilder::new("INSERT INTO shares_models (share_id, model_id) ");
    query_builder.push_values(model_ids.iter(), |mut b, model_id| {
        b.push_bind(share_id);
        b.push_bind(model_id);
    });

    let query = query_builder.build();
    query.execute(db).await?;

    Ok(())
}

pub async fn delete_share(
    db: &DbContext,
    user: &User,
    share_id: &str,
) -> Result<(), DbError> {
    sqlx::query!("DELETE FROM shares WHERE share_id = ? AND share_user_id = ?", share_id, user.id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn create_share(
    db: &DbContext,
    user: &User,
    share_name: &str,
) -> Result<String, DbError> {
    let random_hex = random_hex_32();
    let now = time_now();

    sqlx::query!("
        INSERT INTO shares (share_id, share_user_id, share_created_at, share_name)VALUES (?, ?, ?, ?)", 
        random_hex, 
        user.id, 
        now,
        share_name)
        .execute(db)
        .await?;

    Ok(random_hex)
}

pub async fn rename_share(
    db: &DbContext,
    user: &User,
    share_id: &str,
    new_name: &str,
) -> Result<(), DbError> {
    sqlx::query!("
        UPDATE shares 
        SET share_name = ? 
        WHERE share_id = ? AND share_user_id = ?", 
        new_name, 
        share_id, 
        user.id)
        .execute(db)
        .await?;

    Ok(())
}