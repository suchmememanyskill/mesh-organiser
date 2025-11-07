use db::{blob_db, model::User, model_db};
use tauri::{State, ipc::Response};

use crate::{error::ApplicationError, service::{app_state::AppState, export_service}};

#[tauri::command]
pub async fn get_model_bytes(
    model_id: i64,
    state: State<'_, AppState>,
) -> Result<Response, ApplicationError> {
    let model = model_db::get_models_via_ids(&state.db, &state.get_current_user(), vec![model_id]).await?;

    if model.len() <= 0 {
        return Err(ApplicationError::InternalError(String::from(
            "Failed to find model",
        )));
    }

    let model = &model[0];

    get_blob_bytes(model.blob.sha256.clone(), state).await
}


#[tauri::command]
pub async fn get_blob_bytes(
    sha256: String,
    state: State<'_, AppState>,
) -> Result<Response, ApplicationError> {
    let blob = match blob_db::get_blob_via_sha256(&state.db, &sha256).await? {
        Some(b) => b,
        None => {
            return Err(ApplicationError::InternalError(String::from(
                "Failed to find blob",
            )))
        }
    };

    // Todo: This is not a streamed response. Less efficient than the streaming we did before!
    let bytes = export_service::get_bytes_from_blob(&blob, &state)?;

    Ok(Response::new(bytes))
}