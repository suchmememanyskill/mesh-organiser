use db::{model::{User, hash_password}, user_db};
use tauri::State;

use crate::{error::ApplicationError, tauri_app_state::TauriAppState};

#[tauri::command]
pub async fn get_current_user(
    state: State<'_, TauriAppState>,
) -> Result<User, ApplicationError> {
    let user = state.get_current_user();

    Ok(user)
}

#[tauri::command]
pub async fn set_current_user(
    user_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    state.set_current_user_by_id(user_id).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_users(
    state: State<'_, TauriAppState>,
) -> Result<Vec<User>, ApplicationError> {
    let users = user_db::get_users(&state.app_state.db).await?;

    Ok(users)
}

#[tauri::command]
pub async fn add_user(
    user_name: &str,
    user_email: &str,
    user_password: &str,
    state: State<'_, TauriAppState>,
) -> Result<i64, ApplicationError> {
    let id = user_db::add_user(&state.app_state.db, user_name, user_email, &hash_password(user_password)).await?;

    Ok(id)
}

#[tauri::command]
pub async fn edit_user(
    user_id: i64,
    user_name: &str,
    user_email: &str,
    user_last_sync: Option<String>,
    user_sync_token: Option<String>,
    user_sync_url: Option<String>,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    user_db::edit_user(&state.app_state.db, user_id, user_name, user_email, user_last_sync, user_sync_token, user_sync_url).await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_user(
    user_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    if state.get_current_user().id == user_id {
        return Err(ApplicationError::InternalError("Cannot delete the currently logged in user.".into()));
    }

    // TODO: Check for blobs that maybe orphaned after user deletion?

    user_db::delete_user(&state.app_state.db, user_id).await?;

    Ok(())
}