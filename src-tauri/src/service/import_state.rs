use serde::Serialize;
use strum::Display;
use tauri::{AppHandle, Emitter};

use crate::{error::ApplicationError, service::app_state::AppState};

#[derive(Serialize, Clone)]
pub struct ImportedModelsSet {
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub model_ids: Vec<i64>,
}
#[derive(Serialize, Clone, Display)]
pub enum ImportStatus {
    Idle,
    ProcessingModels,
    FinishedModels,
    ProcessingThumbnails,
    FinishedThumbnails,
    Finished,
    Failure,
}

#[derive(Serialize, Clone)]
pub struct ImportState {
    pub imported_models: Vec<ImportedModelsSet>,
    pub imported_models_count: usize, // TODO: should it be model or models?
    pub model_count: usize,
    pub finished_thumbnails_count: usize,
    pub status: ImportStatus,
    pub origin_url: Option<String>,
    pub failure_reason: Option<String>,
    pub recursive: bool,
    pub delete_after_import: bool,
}

const IMPORT_STATUS_EVENT : &'static str = "import-status";
const IMPORT_MODEL_GROUP_EVENT : &'static str = "import-model-group";
const IMPORT_MODEL_TOTAL_EVENT : &'static str = "import-model-total";
const IMPORT_MODEL_COUNT_EVENT : &'static str = "import-model-count";
const IMPORT_THUMBNAIL_COUNT_EVENT : &'static str = "import-thumbnail-count";
const IMPORT_FAILURE_REASON_EVENT : &'static str = "import-failure-reason";
const IMPORT_ALL_DATA_EVENT : &'static str = "import-all-data";


impl ImportState {
    pub fn new(origin_url: Option<String>, recursive: bool, delete_after_import: bool) -> Self {
        Self {
            imported_models: Vec::new(),
            imported_models_count: 0,
            model_count: 0,
            finished_thumbnails_count: 0,
            status: ImportStatus::Idle,
            failure_reason: None,
            origin_url: origin_url,
            recursive,
            delete_after_import,
        }
    }

    pub fn update_total_model_count(&mut self, count: usize, handle : &AppHandle)
    {
        self.model_count = count;
        let _ = handle.emit(IMPORT_MODEL_TOTAL_EVENT, self.model_count);
    }

    pub fn update_status(&mut self, status: ImportStatus, handle: &AppHandle) 
    {
        self.status = status;
        let _ = handle.emit(IMPORT_STATUS_EVENT, self.status.to_string());
    }

    pub fn set_failure(&mut self, failure_reason: String, handle: &AppHandle)
    {
        self.update_status(ImportStatus::Failure, handle);
        let _ = handle.emit(IMPORT_FAILURE_REASON_EVENT, &failure_reason);
        self.failure_reason = Some(failure_reason);
    }

    pub fn add_new_import_set(&mut self, group_name: Option<String>, handle : &AppHandle)
    {
        if let Some(last) = self.imported_models.last() {
            if last.model_ids.is_empty() {
                return;
            }
        }

        if let Some(group_name) = group_name.clone() {
            let _ = handle.emit(IMPORT_MODEL_GROUP_EVENT, group_name);
        }

        self.imported_models.push(ImportedModelsSet {
            group_id: None,
            group_name: group_name,
            model_ids: Vec::new(),
        });
    }

    pub fn update_finished_thumbnails_count(&mut self, amount: usize, handle : &AppHandle)
    {
        self.finished_thumbnails_count += amount;
        let _ = handle.emit(IMPORT_THUMBNAIL_COUNT_EVENT, self.finished_thumbnails_count);
    }

    pub fn add_model_id_to_current_set(&mut self, model_id: i64, handle : &AppHandle)
    {
        if self.imported_models.is_empty() {
            self.add_new_import_set(None, handle);
        }

        self.imported_models.last_mut().unwrap().model_ids.push(model_id);
        self.imported_models_count = self.imported_models.iter().map(|set| set.model_ids.len()).sum();
        let _ = handle.emit(IMPORT_MODEL_COUNT_EVENT, self.imported_models_count);
    }

    pub fn get_last_group_name(&self) -> Option<String> {
        if let Some(last) = self.imported_models.last() {
            return last.group_name.clone();
        }

        None
    }

    pub fn create_group_from_current_set(&mut self, state: &AppState) -> Result<i64, ApplicationError> {
        if let Some(last) = self.imported_models.last_mut() {
            if last.model_ids.is_empty() {
                return Err(ApplicationError::InternalError("No models to create group from".to_string()));
            }

            if let Some(group_name) = &last.group_name {
                let group_id = crate::db::model_group::add_empty_group_sync(
                    group_name,
                    &state.db,
                );

                crate::db::model_group::set_group_id_on_models_sync(Some(group_id), last.model_ids.clone(), &state.db);

                last.group_id = Some(group_id);
                return Ok(group_id);
            }

            return Err(ApplicationError::InternalError("Group has no name".to_string()));
        } else {
            return Err(ApplicationError::InternalError("No models to create group from".to_string()));
        }
    }

    pub fn emit_all(&self, handle: &AppHandle) {
        let _ = handle.emit(IMPORT_STATUS_EVENT, self.status.to_string());
        let _ = handle.emit(IMPORT_MODEL_COUNT_EVENT, self.imported_models_count);
        let _ = handle.emit(IMPORT_THUMBNAIL_COUNT_EVENT, self.finished_thumbnails_count);
        let _ = handle.emit(IMPORT_MODEL_TOTAL_EVENT, self.model_count);

        if let Some(last) = self.imported_models.last() {
            if let Some(group_name) = &last.group_name {
                let _ = handle.emit(IMPORT_MODEL_GROUP_EVENT, group_name);
            }
        }

        if let Some(failure_reason) = &self.failure_reason {
            let _ = handle.emit(IMPORT_FAILURE_REASON_EVENT, failure_reason);
        }
    }

    // Pushes all data to the frontend
    pub fn push_all_data_to_frontend(&self, handle: &AppHandle)
    {
        let _ = handle.emit(IMPORT_ALL_DATA_EVENT, self);
    }
}