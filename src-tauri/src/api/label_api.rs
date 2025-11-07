use db::{label_db, model::Label};
use tauri::State;

use crate::{error::ApplicationError, service::app_state::AppState};


#[tauri::command]
pub async fn get_labels(include_ungrouped_models: Option<bool>, state: State<'_, AppState>) -> Result<Vec<Label>, ApplicationError> {
    let labels = label_db::get_labels(&state.db, &state.get_current_user(), include_ungrouped_models.unwrap_or(false))
        .await?;

    Ok(labels)
}

#[tauri::command]
pub async fn add_label(
    label_name: &str,
    label_color: i64,
    state: State<'_, AppState>,
) -> Result<i64, ApplicationError> {
    let id = label_db::add_label(&state.db, &state.get_current_user(), label_name, label_color, true)
        .await?;

    Ok(id)
}

#[tauri::command]
pub async fn set_labels_on_model(
    label_ids: Vec<i64>,
    model_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::remove_all_labels_from_models(&state.db, &state.get_current_user(), &[model_id], true).await?;
    label_db::add_labels_on_models(&state.db, &state.get_current_user(), &label_ids, &[model_id], true).await?;

    Ok(())
}

#[tauri::command]
pub async fn set_label_on_models(
    label_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::remove_labels_from_models(&state.db, &state.get_current_user(), &[label_id], &model_ids, true).await?;
    label_db::add_labels_on_models(&state.db, &state.get_current_user(), &[label_id], &model_ids, true).await?;

    Ok(())
}

#[tauri::command]
pub async fn remove_label_from_models(
    label_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::remove_labels_from_models(&state.db, &state.get_current_user(), &[label_id], &model_ids, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn edit_label(
    label_id: i64,
    label_name: &str,
    label_color: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::edit_label(&state.db, &state.get_current_user(), label_id, label_name, label_color, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_label(label_id: i64, state: State<'_, AppState>) -> Result<(), ApplicationError> {
    label_db::delete_label(&state.db, &state.get_current_user(), label_id, true)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn add_childs_to_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::add_childs_to_label(&state.db, &state.get_current_user(), parent_label_id, child_label_ids, true)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn remove_childs_from_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::remove_childs_from_label(&state.db, &state.get_current_user(), parent_label_id, child_label_ids, true)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn set_childs_on_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    label_db::remove_all_childs_from_label(&state.db, &state.get_current_user(), parent_label_id, true)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;
    
    if !child_label_ids.is_empty() {
        label_db::add_childs_to_label(&state.db, &state.get_current_user(), parent_label_id, child_label_ids, true)
            .await
            .map_err(|e| ApplicationError::InternalError(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_keywords_on_label(
    label_id: i64,
    keywords: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label_keyword_db::set_keywords_for_label(&state.db, &state.get_current_user(), label_id, keywords, true)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn get_keywords_for_label(
    label_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<db::model::LabelKeyword>, ApplicationError> {
    let keywords = db::label_keyword_db::get_keywords_for_label(&state.db, &state.get_current_user(), label_id)
        .await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

    Ok(keywords)
}