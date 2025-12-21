use db::{
    model::{User, hash_password},
    user_db,
};
use service::export_service;
use tauri::State;

use crate::{error::ApplicationError, tauri_app_state::TauriAppState};

#[tauri::command]
pub async fn get_current_user(state: State<'_, TauriAppState>) -> Result<User, ApplicationError> {
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
pub async fn get_users(state: State<'_, TauriAppState>) -> Result<Vec<User>, ApplicationError> {
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
    let id = user_db::add_user(
        &state.app_state.db,
        user_name,
        user_email,
        &hash_password(user_password),
    )
    .await?;

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
    user_db::edit_user_min(
        &state.app_state.db,
        user_id,
        user_name,
        user_email,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_last_sync_time(
    user_id: i64,
    user_last_sync: &str,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    user_db::edit_user_last_sync_time(
        &state.app_state.db,
        user_id,
        user_last_sync,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_sync_state(
    user_sync_token: String,
    user_sync_url: String,
    online: bool,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    user_db::set_user_sync_token(
        &state.app_state.db,
        state.get_current_user().id,
        &user_sync_token,
        &user_sync_url,
        online,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn unset_sync_state(
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    user_db::clear_user_sync(&state.app_state.db, state.get_current_user().id).await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_user(
    user_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    if state.get_current_user().id == user_id {
        return Err(ApplicationError::InternalError(
            "Cannot delete the currently logged in user.".into(),
        ));
    }

    user_db::delete_user(&state.app_state.db, user_id).await?;

    export_service::delete_dead_blobs(&state.app_state).await?;

    Ok(())
}
