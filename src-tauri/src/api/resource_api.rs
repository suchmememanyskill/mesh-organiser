use db::{model::{ModelGroup, Resource, ResourceFlags, ResourceMeta, random_hex_32, time_now}, resource_db};
use service::resource_service;
use tauri::State;

use crate::{api, error::ApplicationError, tauri_app_state::TauriAppState};

#[tauri::command]
pub async fn get_resources(state: State<'_, TauriAppState>) -> Result<Vec<ResourceMeta>, ApplicationError> {
    let resources = resource_db::get_resources(&state.app_state.db, &state.get_current_user())
        .await?;

    Ok(resources)
}

#[tauri::command]
pub async fn add_resource(
    resource_name: &str,
    state: State<'_, TauriAppState>,
) -> Result<ResourceMeta, ApplicationError> {
    let id = resource_db::add_resource(&state.app_state.db, &state.get_current_user(), resource_name, true)
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
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    resource_db::edit_resource(&state.app_state.db, &state.get_current_user(), resource_id, resource_name, resource_flags, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_resource(
    resource_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    let user = state.get_current_user();
    let resource = resource_db::get_resource_meta_by_id(&state.app_state.db, &user, resource_id)
        .await?;

    if resource.is_none() {
        return Err(ApplicationError::InternalError(String::from(
            "Resource not found",
        )));
    }

    let resource = resource.unwrap();

    resource_service::delete_resource_folder(&resource, &user, &state.app_state).await?;
    resource_db::delete_resource(&state.app_state.db, &state.get_current_user(), resource.id, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn open_resource_folder(
    resource_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    let user = state.get_current_user();
    let resource = resource_db::get_resource_meta_by_id(&state.app_state.db, &user, resource_id)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

    if resource.is_none() {
        return Err(ApplicationError::InternalError(String::from(
            "Resource not found",
        )));
    }

    let resource = resource.unwrap();

    resource_service::open_resource_folder(&resource, &user, &state.app_state).await?;
    Ok(())
}

#[tauri::command]
pub async fn set_resource_on_group(
    resource_id: Option<i64>,
    group_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    resource_db::set_resource_on_group(&state.app_state.db, &state.get_current_user(), resource_id, group_id, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_groups_for_resource(
    resource_id: i64,
    state: State<'_, TauriAppState>,
) -> Result<Vec<ModelGroup>, ApplicationError> {
    let groups = resource_db::get_groups_for_resource(&state.app_state.db, &state.get_current_user(), resource_id)
        .await?;

    Ok(groups)
}