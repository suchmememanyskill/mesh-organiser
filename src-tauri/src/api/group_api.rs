use std::str::FromStr;

use db::{group_db::GroupOrderBy, model::{ModelGroup, User}};
use tauri::State;

use crate::{error::ApplicationError, service::app_state::AppState};

#[tauri::command]
pub async fn get_groups(
    group_ids : Option<Vec<i64>>,
    label_ids: Option<Vec<i64>>,
    order_by: Option<String>,
    text_search: Option<String>,
    page: u32,
    page_size: u32,
    include_ungrouped_models : Option<bool>,
    state: State<'_, AppState>
) -> Result<Vec<ModelGroup>, ApplicationError> {
    let groups = db::group_db::get_groups(&state.db, &state.get_current_user(), db::group_db::GroupFilterOptions {
        group_ids,
        label_ids,
        order_by: order_by.map(|s| GroupOrderBy::from_str(&s).unwrap_or(GroupOrderBy::NameAsc)),
        text_search,
        page,
        page_size,
        include_ungrouped_models: include_ungrouped_models.unwrap_or(false),
    }).await?;

    Ok(groups.items)
}

#[tauri::command]
pub async fn ungroup(group_id: i64, state: State<'_, AppState>) -> Result<(), ApplicationError> {
    db::group_db::delete_group(&state.db, &state.get_current_user(), group_id, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn add_group(group_name: &str, state: State<'_, AppState>) -> Result<i64, ApplicationError> {
    let id = db::group_db::add_empty_group(&state.db, &state.get_current_user(), group_name, true)
        .await?;

    Ok(id)
}

#[tauri::command]
pub async fn add_models_to_group(
    group_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::group_db::set_group_id_on_models(&state.db, &state.get_current_user(), Some(group_id), model_ids, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_models_from_group(
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::group_db::set_group_id_on_models(&state.db, &state.get_current_user(), None, model_ids, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn edit_group(
    group_id: i64,
    group_name: &str,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::group_db::edit_group(&state.db, &state.get_current_user(), group_id, group_name, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_dead_groups(state: State<'_, AppState>) -> Result<(), ApplicationError> {
    db::group_db::delete_dead_groups(&state.db)
        .await?;

    Ok(())
}