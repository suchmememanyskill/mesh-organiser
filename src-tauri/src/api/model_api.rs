use std::path::PathBuf;
use std::str::FromStr;

use db::blob_db;
use db::model::{ModelFlags, User};
use db::model_db::{self, ModelFilterOptions, ModelOrderBy};
use serde::Serialize;
use service::{export_service, import_service};
use service::import_state::ImportStatus;
use tauri::{AppHandle, State};
use crate::error::ApplicationError;
use crate::{TauriAppState, tauri_thumbnail_service};
use crate::ImportState;
use crate::tauri_import_state::import_state_new_tauri;

#[tauri::command]
pub async fn add_model(
    path: &str,
    recursive: bool,
    delete_imported: bool,
    origin_url: Option<String>,
    open_in_slicer: bool,
    state: State<'_, TauriAppState>,
    app_handle: AppHandle,
) -> Result<ImportState, ApplicationError> {
    let path_clone = String::from(path);
    let state_clone = state.clone();
    let mut import_state = import_state_new_tauri(origin_url, recursive, delete_imported, &state, &app_handle);
    import_state = import_service::import_path(&path_clone, &state_clone.app_state, import_state).await?;

    let model_ids: Vec<i64> = import_state
        .imported_models
        .iter()
        .flat_map(|f| f.model_ids.clone())
        .collect();

    let models = model_db::get_models_via_ids(&state.app_state.db, &state.get_current_user(), model_ids).await?;
    tauri_thumbnail_service::generate_thumbnails(
        &models,
        &state,
        &app_handle,
        false,
        &mut import_state,
    )
    .await?;

    if open_in_slicer && models.len() > 0 {
        if let Some(slicer) = &state.get_configuration().slicer {
            slicer.open(models, &state.app_state).await?;
        }
    }

    println!("Import finished: {:?}", import_state);
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
    model_flags: Option<ModelFlags>,
    page: u32,
    page_size: u32,
    state: State<'_, TauriAppState>
) -> Result<Vec<db::model::Model>, ApplicationError> {
    let models = model_db::get_models(&state.app_state.db, &state.get_current_user(), ModelFilterOptions {
        model_ids,
        group_ids,
        label_ids,
        order_by: order_by.map(|s| ModelOrderBy::from_str(&s).unwrap_or(ModelOrderBy::AddedDesc)),
        model_flags,
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
    model_flags: Option<ModelFlags>,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    db::model_db::edit_model(
        &state.app_state.db,
        &state.get_current_user(),
        model_id,
        model_name,
        model_url,
        model_description,
        model_flags.unwrap_or(ModelFlags::empty()),
        None,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_model(model_id: i64, state: State<'_, TauriAppState>) -> Result<(), ApplicationError> {
    let model = model_db::get_models_via_ids(&state.app_state.db, &state.get_current_user(), vec![model_id]).await?;

    if model.len() != 1 {
        return Err(ApplicationError::InternalError(String::from(
            "Failed to find model to delete",
        )));
    }

    let model = &model[0];

    model_db::delete_model(&state.app_state.db, &state.get_current_user(), model_id)
        .await?;

    if blob_db::get_blob_model_usage_count(&state.app_state.db, model.blob.id).await? <= 0 {
        let model_path =
            PathBuf::from(state.get_model_dir()).join(format!("{}.{}", model.blob.sha256, model.blob.filetype));
        let image_path = PathBuf::from(state.get_image_dir()).join(format!("{}.png", model.blob.sha256));

        if model_path.exists() {
            std::fs::remove_file(model_path)?;
        }

        if image_path.exists() {
            std::fs::remove_file(image_path)?;
        }

        blob_db::delete_blob(&state.app_state.db, model.blob.id).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_model_count(flags : Option<ModelFlags>, state: State<'_, TauriAppState>) -> Result<usize, ApplicationError> {
    let count = db::model_db::get_model_count(&state.app_state.db, &state.get_current_user(), flags).await?;

    Ok(count)
}

#[derive(Debug, Serialize)]
pub struct ModelDiskSpaceUsage {
    pub size_uncompressed: u64,
    pub size_compressed: u64,
}

#[tauri::command]
pub async fn get_model_disk_space_usage(state: State<'_, TauriAppState>) -> Result<ModelDiskSpaceUsage, ApplicationError> {
    let data = model_db::get_size_of_models(&state.app_state.db, &state.get_current_user()).await?;
    let local = export_service::get_size_of_blobs(&data.blob_sha256, &state.app_state)?;

    Ok(ModelDiskSpaceUsage {
        size_uncompressed: data.total_size as u64,
        size_compressed: local,
    })
}