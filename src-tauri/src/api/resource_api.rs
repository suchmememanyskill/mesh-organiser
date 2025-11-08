use db::{model::{ModelGroup, Resource, ResourceFlags, ResourceMeta, random_hex_32, time_now}, resource_db};
use tauri::State;

use crate::{error::ApplicationError, service::{app_state::AppState, resource_service}};

#[tauri::command]
pub async fn get_resources(state: State<'_, AppState>) -> Result<Vec<ResourceMeta>, ApplicationError> {
    let resources = resource_db::get_resources(&state.db, &state.get_current_user())
        .await?;

    Ok(resources)
}

#[tauri::command]
pub async fn add_resource(
    resource_name: &str,
    state: State<'_, AppState>,
) -> Result<ResourceMeta, ApplicationError> {
    let id = resource_db::add_resource(&state.db, &state.get_current_user(), resource_name, true)
        .await?;

    Ok(ResourceMeta {
        id: id,
        name: resource_name.to_string(),
        flags: ResourceFlags::empty(),
        created: time_now(),
        unique_global_id: random_hex_32(),
    })
}

#[tauri::command]
pub async fn edit_resource(
    resource_id: i64,
    resource_name: &str,
    resource_flags: ResourceFlags,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    resource_db::edit_resource(&state.db, &state.get_current_user(), resource_id, resource_name, resource_flags, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_resource(
    resource_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let user = state.get_current_user();
    let resource = resource_db::get_resource_meta_by_id(&state.db, &user, resource_id)
        .await?;

    if resource.is_none() {
        return Err(ApplicationError::InternalError(String::from(
            "Resource not found",
        )));
    }

    let resource = resource.unwrap();

    resource_service::delete_resource_folder(&resource, &user, &state).await?;
    resource_db::delete_resource(&state.db, &state.get_current_user(), resource.id, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn open_resource_folder(
    resource_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let user = state.get_current_user();
    let resource = resource_db::get_resource_meta_by_id(&state.db, &user, resource_id)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

    if resource.is_none() {
        return Err(ApplicationError::InternalError(String::from(
            "Resource not found",
        )));
    }

    let resource = resource.unwrap();

    resource_service::open_resource_folder(&resource, &user, &state).await?;
    Ok(())
}

#[tauri::command]
pub async fn set_resource_on_group(
    resource_id: Option<i64>,
    group_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    resource_db::set_resource_on_group(&state.db, &state.get_current_user(), resource_id, group_id, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_groups_for_resource(
    resource_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ModelGroup>, ApplicationError> {
    let groups = resource_db::get_groups_for_resource(&state.db, &state.get_current_user(), resource_id)
        .await?;

    Ok(groups)
}