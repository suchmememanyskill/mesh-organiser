use std::fmt::Debug;

use serde::Serialize;
use strum::Display;
use tauri::{AppHandle, Emitter};

use crate::{error::ApplicationError, service::app_state::AppState};

#[derive(Serialize, Clone, Debug)]
pub struct ImportedModelsSet {
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub model_ids: Vec<i64>,
}
#[derive(Serialize, Debug, Display)]
pub enum ImportStatus {
    Idle,
    ProcessingModels,
    FinishedModels,
    ProcessingThumbnails,
    FinishedThumbnails,
    Finished,
    Failure,
}

#[derive(Serialize, Debug)]
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
    
    #[serde(skip)]
    pub emitter: Box<dyn ImportStateEmitter + Send + Sync>,
}

impl Debug for dyn ImportStateEmitter + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImportStateEmitter")
    }
}

const IMPORT_STATUS_EVENT: &'static str = "import-status";
const IMPORT_MODEL_GROUP_EVENT: &'static str = "import-model-group";
const IMPORT_MODEL_TOTAL_EVENT: &'static str = "import-model-total";
const IMPORT_MODEL_COUNT_EVENT: &'static str = "import-model-count";
const IMPORT_THUMBNAIL_COUNT_EVENT: &'static str = "import-thumbnail-count";
const IMPORT_FAILURE_REASON_EVENT: &'static str = "import-failure-reason";
const IMPORT_ALL_DATA_EVENT: &'static str = "import-all-data";

pub trait ImportStateEmitter {
    fn status_event(&self, status: &ImportState);
    fn model_group_event(&self, status: &ImportState);
    fn model_total_event(&self, status: &ImportState);
    fn model_count_event(&self, status: &ImportState);
    fn thumbnail_count_event(&self, status: &ImportState);
    fn failure_reason_event(&self, status: &ImportState);
    fn all_data_event(&self, state: &ImportState);
}

pub struct NoneImportStateEmitter;

impl ImportStateEmitter for NoneImportStateEmitter {
    fn status_event(&self, _status: &ImportState) {}
    fn model_total_event(&self, _status: &ImportState) {}
    fn failure_reason_event(&self, _status: &ImportState) {}
    fn model_group_event(&self, _status: &ImportState) {}
    fn thumbnail_count_event(&self, _status: &ImportState) {}
    fn model_count_event(&self, _status: &ImportState) {}
    fn all_data_event(&self, _state: &ImportState) {}
}

pub struct TauriImportStateEmitter {
    handle: AppHandle,
}

impl ImportStateEmitter for TauriImportStateEmitter {
    fn status_event(&self, status: &ImportState) {
        let _ = self.handle.emit(IMPORT_STATUS_EVENT, status.status.to_string());
    }
    
    fn model_total_event(&self, status: &ImportState) {
        let _ = self.handle.emit(IMPORT_MODEL_TOTAL_EVENT, status.model_count);
    }

    fn failure_reason_event(&self, status: &ImportState) {
        if let Some(reason) = &status.failure_reason {
            let _ = self.handle.emit(IMPORT_FAILURE_REASON_EVENT, reason);
        }
    }

    fn model_group_event(&self, status: &ImportState) {
        if let Some(group_name) = status.get_last_group_name() {
            let _ = self.handle.emit(IMPORT_MODEL_GROUP_EVENT, group_name);
        }
    }

    fn thumbnail_count_event(&self, status: &ImportState) {
        let _ = self.handle.emit(IMPORT_THUMBNAIL_COUNT_EVENT, status.finished_thumbnails_count);
    }

    fn model_count_event(&self, status: &ImportState) {
        let _ = self.handle.emit(IMPORT_MODEL_COUNT_EVENT, status.imported_models_count);
    }

    fn all_data_event(&self, state: &ImportState) {
        let _ = self.handle.emit(IMPORT_ALL_DATA_EVENT, state);
    }
}

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
            emitter: Box::new(NoneImportStateEmitter {}),
        }
    }

    pub fn new_tauri(origin_url: Option<String>, recursive: bool, delete_after_import: bool, handle: &AppHandle) -> Self {
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
            emitter: Box::new(TauriImportStateEmitter { handle: handle.clone() }),
        }
    }

    pub fn update_total_model_count(&mut self, count: usize) {
        self.model_count = count;
        self.emitter.model_total_event(self);
    }

    pub fn update_status(&mut self, status: ImportStatus) {
        self.status = status;
        self.emitter.status_event(self);
    }

    pub fn set_failure(&mut self, failure_reason: String) {
        self.update_status(ImportStatus::Failure);
        self.failure_reason = Some(failure_reason);
        self.emitter.failure_reason_event(self);
    }

    pub fn add_new_import_set(&mut self, group_name: Option<String>) {
        if let Some(last) = self.imported_models.last() {
            if last.model_ids.is_empty() {
                return;
            }
        }

        self.imported_models.push(ImportedModelsSet {
            group_id: None,
            group_name: group_name,
            model_ids: Vec::new(),
        });

        self.emitter.model_group_event(self);
    }

    pub fn update_finished_thumbnails_count(&mut self, amount: usize) {
        self.finished_thumbnails_count += amount;
        self.emitter.thumbnail_count_event(self);
    }

    pub fn add_model_id_to_current_set(&mut self, model_id: i64) {
        if self.imported_models.is_empty() {
            self.add_new_import_set(None);
        }

        self.imported_models
            .last_mut()
            .unwrap()
            .model_ids
            .push(model_id);
        self.imported_models_count = self
            .imported_models
            .iter()
            .map(|set| set.model_ids.len())
            .sum();
        self.emitter.model_count_event(self);
    }

    pub fn get_last_group_name(&self) -> Option<String> {
        if let Some(last) = self.imported_models.last() {
            return last.group_name.clone();
        }

        None
    }

    pub fn create_group_from_current_set(
        &mut self,
        state: &AppState,
    ) -> Result<i64, ApplicationError> {
        if let Some(last) = self.imported_models.last_mut() {
            if last.model_ids.is_empty() {
                return Err(ApplicationError::InternalError(
                    "No models to create group from".to_string(),
                ));
            }

            if let Some(group_name) = &last.group_name {
                let group_id = tauri::async_runtime::block_on(async {
                    db::group_db::add_empty_group(&state.db, &state.get_current_user(), group_name, true)
                        .await
                })?;

                tauri::async_runtime::block_on(async {
                    db::group_db::set_group_id_on_models(
                        &state.db,
                        &state.get_current_user(),
                        Some(group_id),
                        last.model_ids.clone(),
                        true,
                    )
                    .await
                })?;

                last.group_id = Some(group_id);
                return Ok(group_id);
            }

            return Err(ApplicationError::InternalError(
                "Group has no name".to_string(),
            ));
        } else {
            return Err(ApplicationError::InternalError(
                "No models to create group from".to_string(),
            ));
        }
    }

    pub fn emit_all(&self) {
        self.emitter.status_event(self);
        self.emitter.model_count_event(self);
        self.emitter.thumbnail_count_event(self);
        self.emitter.model_total_event(self);
        self.emitter.model_group_event(self);
        self.emitter.failure_reason_event(self);
    }

    // Pushes all data to the frontend
    pub fn push_all_data_to_frontend(&self, handle: &AppHandle) {
        let _ = handle.emit(IMPORT_ALL_DATA_EVENT, self);
    }
}
