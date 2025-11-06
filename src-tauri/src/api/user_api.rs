use db::{model::User, user_db};
use tauri::State;

use crate::{error::ApplicationError, service::app_state::AppState};


#[tauri::command]
pub async fn get_current_user(
    state: State<'_, AppState>,
) -> Result<User, ApplicationError> {
    let user = state.get_current_user();

    Ok(user)
}

#[tauri::command]
pub async fn set_current_user(
    user_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    state.set_current_user_by_id(user_id).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_users(
    state: State<'_, AppState>,
) -> Result<Vec<User>, ApplicationError> {
    let users = user_db::get_users(&state.db).await?;

    Ok(users)
}

#[tauri::command]
pub async fn add_user(
    user_name: &str,
    state: State<'_, AppState>,
) -> Result<i64, ApplicationError> {
    let id = user_db::add_user(&state.db, user_name, "", "").await?;

    Ok(id)
}

#[tauri::command]
pub async fn edit_user(
    user_id: i64,
    user_name: &str,
    user_last_sync: Option<String>,
    user_sync_token: Option<String>,
    user_sync_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    user_db::edit_user(&state.db, user_id, user_name, "".into(), "".into(), user_last_sync, user_sync_token, user_sync_url).await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_user(
    user_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    user_db::delete_user(&state.db, user_id).await?;

    Ok(())
}