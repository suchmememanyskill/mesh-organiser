use std::fmt::Debug;

use db::{group_db, model::User};
use serde::Serialize;
use strum::Display;

use crate::{app_state::AppState, service_error::ServiceError};

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
    pub user: User,
    
    #[serde(skip)]
    pub emitter: Box<dyn ImportStateEmitter + Send + Sync>,
}

impl Debug for dyn ImportStateEmitter + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImportStateEmitter")
    }
}

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

impl ImportState {
    pub fn new(origin_url: Option<String>, recursive: bool, delete_after_import : bool, user: User) -> Self {
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
            user: user,
        }
    }

    pub fn new_with_emitter(origin_url: Option<String>, recursive: bool, delete_after_import: bool, user: User, emitter: Box<dyn ImportStateEmitter + Send + Sync>) -> Self {
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
            emitter: emitter,
            user: user
        }
    }

    pub fn set_emitter(&mut self, emitter: Box<dyn ImportStateEmitter + Send + Sync>) {
        self.emitter = emitter;
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
        if let Some(last) = self.imported_models.last_mut() {
            if last.model_ids.is_empty() {
                last.group_name = group_name;
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

    pub async fn create_groups_from_all_sets(
        &mut self,
        state: &AppState,
    ) -> Result<Vec<i64>, ServiceError> {
        let mut ids = Vec::new();
        let user = &self.user;
        for set in self.imported_models.iter_mut() {
            if set.group_id.is_some() || set.group_name.is_none() || set.model_ids.is_empty() {
                continue;
            }

            let group_name = set.group_name.as_ref().unwrap();
            let group_id = group_db::add_empty_group(&state.db, user, group_name, None).await?;

            group_db::set_group_id_on_models(
                    &state.db,
                    user,
                    Some(group_id),
                    set.model_ids.clone(),
                    None,
                )
                .await?;

            set.group_id = Some(group_id);
            ids.push(group_id);
        }

        Ok(ids)
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
    pub fn push_all_data_to_frontend(&self) {
        self.emitter.all_data_event(self);
    }
}
