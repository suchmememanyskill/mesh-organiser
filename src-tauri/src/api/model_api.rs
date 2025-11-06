use std::path::PathBuf;
use std::str::FromStr;

use db::model::{ModelFlags, User};
use db::model_db::{self, ModelOrderBy};
use tauri::{AppHandle, State};
use crate::error::ApplicationError;
use crate::service::app_state::AppState;
use crate::service::{self, import_service};
use crate::service::import_state::{ImportState, ImportStatus};

#[tauri::command]
pub async fn add_model(
    path: &str,
    recursive: bool,
    delete_imported: bool,
    origin_url: Option<String>,
    open_in_slicer: bool,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ImportState, ApplicationError> {
    let path_clone = String::from(path);
    let state_clone = state.real_clone();
    let handle_clone = app_handle.clone();
    let mut import_state = ImportState::new(origin_url, recursive, delete_imported);

    import_state = tauri::async_runtime::spawn_blocking(move || {
        let _lock = state_clone.import_mutex.lock().unwrap();
        import_service::import_path(&path_clone, &state_clone, &handle_clone, &mut import_state)?;

        Result::<ImportState, ApplicationError>::Ok(import_state)
    })
    .await
    .unwrap()?;

    let model_ids: Vec<i64> = import_state
        .imported_models
        .iter()
        .flat_map(|f| f.model_ids.clone())
        .collect();
    let models = model_db::get_models_via_ids(&state.db, &state.get_current_user(), model_ids).await?;
    service::thumbnail_service::generate_thumbnails(
        &models,
        &state,
        &app_handle,
        false,
        &mut import_state,
    )
    .await?;

    if open_in_slicer && models.len() > 0 {
        if let Some(slicer) = &state.get_configuration().slicer {
            slicer.open(models, &state)?;
        }
    }

    import_state.status = ImportStatus::Finished;
    Ok(import_state)
}

#[tauri::command]
pub async fn get_models(
    model_ids: Option<Vec<i64>>,
    group_ids: Option<Vec<i64>>,
    label_ids: Option<Vec<i64>>,
    order_by: Option<String>,
    text_search: Option<String>,
    page: u32,
    page_size: u32,
    state: State<'_, AppState>
) -> Result<Vec<db::model::Model>, ApplicationError> {
    let models = db::model_db::get_models(&state.db, &state.get_current_user(), db::model_db::ModelFilterOptions {
        model_ids,
        group_ids,
        label_ids,
        order_by: order_by.map(|s| ModelOrderBy::from_str(&s).unwrap_or(ModelOrderBy::AddedDesc)),
        text_search,
        page,
        page_size,
    }).await?;

    Ok(models.items)
}

#[tauri::command]
pub async fn edit_model(
    model_id: i64,
    model_name: &str,
    model_url: Option<&str>,
    model_description: Option<&str>,
    model_flags: ModelFlags,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model_db::edit_model(
        &state.db,
        &state.get_current_user(),
        model_id,
        model_name,
        model_url,
        model_description,
        model_flags,
        true,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_model(model_id: i64, state: State<'_, AppState>) -> Result<(), ApplicationError> {
    let model = model_db::get_models_via_ids(&state.db, &state.get_current_user(), vec![model_id]).await?;

    if model.len() != 1 {
        return Err(ApplicationError::InternalError(String::from(
            "Failed to find model to delete",
        )));
    }

    let model = &model[0];

    db::model_db::delete_model(&state.db, &state.get_current_user(), model_id, true)
        .await?;

    // TODO: Split this off into a managed layer between server and desktop app
    // TODO: This should happen on a blob level, not on a model level
    let model_path =
        PathBuf::from(state.get_model_dir()).join(format!("{}.{}", model.blob.sha256, model.blob.filetype));
    let image_path = PathBuf::from(state.get_image_dir()).join(format!("{}.png", model.blob.sha256));

    if model_path.exists() {
        std::fs::remove_file(model_path)?;
    }

    if image_path.exists() {
        std::fs::remove_file(image_path)?;
    }

    Ok(())
}