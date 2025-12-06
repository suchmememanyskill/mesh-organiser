use service::import_state::{ImportState, ImportStateEmitter};
use tauri::{AppHandle, Emitter};

use crate::tauri_app_state::TauriAppState;

const IMPORT_STATUS_EVENT: &'static str = "import-status";
const IMPORT_MODEL_GROUP_EVENT: &'static str = "import-model-group";
const IMPORT_MODEL_TOTAL_EVENT: &'static str = "import-model-total";
const IMPORT_MODEL_COUNT_EVENT: &'static str = "import-model-count";
const IMPORT_THUMBNAIL_COUNT_EVENT: &'static str = "import-thumbnail-count";
const IMPORT_FAILURE_REASON_EVENT: &'static str = "import-failure-reason";
const IMPORT_ALL_DATA_EVENT: &'static str = "import-all-data";

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


pub fn import_state_new_tauri(
    origin_url: Option<String>,
    recursive: bool,
    delete_after_import: bool,
    import_as_path: bool,
    app_state: &TauriAppState,
    app_handle: &AppHandle,
) -> ImportState {
    ImportState::new_with_emitter(origin_url, recursive, delete_after_import, import_as_path, app_state.get_current_user(), Box::new(TauriImportStateEmitter {
        handle: app_handle.clone(),
    }))
}

