use std::{str::FromStr, time::Instant};

use db::{group_db::GroupOrderBy, model::{ModelGroup, ModelGroupMeta, User}, random_hex_32, time_now};
use tauri::State;

use crate::{error::ApplicationError, tauri_app_state::TauriAppState};

#[tauri::command]
pub async fn get_groups(
    model_ids: Option<Vec<i64>>,
    group_ids : Option<Vec<i64>>,
    label_ids: Option<Vec<i64>>,
    order_by: Option<String>,
    text_search: Option<String>,
    page: u32,
    page_size: u32,
    include_ungrouped_models : Option<bool>,
    state: State<'_, TauriAppState>
) -> Result<Vec<ModelGroup>, ApplicationError> {
    let instant = Instant::now();
    let groups = db::group_db::get_groups(&state.app_state.db, &state.get_current_user(), db::group_db::GroupFilterOptions {
        model_ids,
        group_ids,
        label_ids,
        order_by: order_by.map(|s| GroupOrderBy::from_str(&s).unwrap_or(GroupOrderBy::NameAsc)),
        text_search,
        page,
        page_size,
        include_ungrouped_models: include_ungrouped_models.unwrap_or(false),
    }).await?;

    println!("get_groups took {:?}", instant.elapsed());

    Ok(groups.items)
}

#[tauri::command]
pub async fn ungroup(group_id: i64, state: State<'_, TauriAppState>) -> Result<(), ApplicationError> {
    db::group_db::delete_group(&state.app_state.db, &state.get_current_user(), group_id)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn add_group(group_name: &str, state: State<'_, TauriAppState>) -> Result<ModelGroupMeta, ApplicationError> {
    let id = db::group_db::add_empty_group(&state.app_state.db, &state.get_current_user(), group_name, None)
        .await?;

    Ok(ModelGroupMeta {
        id,
        name: group_name.to_string(),
        created: time_now(),
        unique_global_id: random_hex_32(),
        resource_id: None,
        last_modified: time_now(),
    })
}

#[tauri::command]
pub async fn add_models_to_group(
    group_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    db::group_db::set_group_id_on_models(&state.app_state.db, &state.get_current_user(), Some(group_id), model_ids, None)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_models_from_group(
    model_ids: Vec<i64>,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    db::group_db::set_group_id_on_models(&state.app_state.db, &state.get_current_user(), None, model_ids, None)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn edit_group(
    group_id: i64,
    group_name: &str,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    db::group_db::edit_group(&state.app_state.db, &state.get_current_user(), group_id, group_name, None)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_dead_groups(state: State<'_, TauriAppState>) -> Result<(), ApplicationError> {
    db::group_db::delete_dead_groups(&state.app_state.db)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_group_count(include_ungrouped_models : Option<bool>, state: State<'_, TauriAppState>) -> Result<usize, ApplicationError> {
    let count = db::group_db::get_group_count(&state.app_state.db, &state.get_current_user(), include_ungrouped_models.unwrap_or(false))
        .await?;

    Ok(count)
}