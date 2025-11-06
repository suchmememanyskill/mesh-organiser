use db::db_context::DbContext;
use db::model::User;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_deep_link::DeepLinkExt;

use crate::configuration;
use configuration::{Configuration, StoredConfiguration, stored_to_configuration};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Serialize)]
pub struct InitialState {
    pub deep_link_url: Option<String>,
    pub max_parallelism: usize,
    pub collapse_sidebar: bool,
}

pub struct AppState {
    pub db: Arc<DbContext>,
    pub configuration: Mutex<Configuration>,
    pub import_mutex: Arc<Mutex<()>>,
    pub initial_state: InitialState,
    pub app_data_path: String,
    pub current_user: Arc<Mutex<User>>,
}

impl AppState {
    pub fn get_current_user(&self) -> User {
        let user = self.current_user.lock().unwrap();
        user.clone()
    }

    pub async fn set_current_user_by_id(&self, user_id: i64) -> Result<(), crate::error::ApplicationError> {
        let user = db::user_db::get_user_by_id(&self.db, user_id).await?;

        if user.is_none() {
            return Err(crate::error::ApplicationError::InternalError("User not found".into()));
        }

        let mut current_user = self.current_user.lock().unwrap();
        *current_user = user.unwrap();

        let mut configuration = self.configuration.lock().unwrap();
        configuration.last_user_id = user_id;

        Ok(())
    }

    // TODO: Change to pathbuf
    pub fn get_model_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.get_configuration().data_path.clone());
        path_buff.push("models");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create model directory");
        }

        String::from(path_buff.to_str().unwrap())
    }

    // TODO: Change to pathbuf
    pub fn get_image_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.app_data_path.clone());
        path_buff.push("images");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create image directory");
        }

        String::from(path_buff.to_str().unwrap())
    }

    pub fn get_resources_dir(&self) -> PathBuf {
        let mut path_buff = PathBuf::from(self.get_configuration().data_path.clone());
        path_buff.push("resources");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone())
                .expect("Failed to create resources directory");
        }

        path_buff
    }

    pub fn write_configuration(&self, new_configuration: &Configuration) -> bool {
        // TODO: This should have all settings, not just a select few
        // TODO: We should probably split up settings per section anyway
        let path = PathBuf::from(self.app_data_path.clone());
        let path = path.join("settings.json");

        let json = serde_json::to_string(&new_configuration).unwrap();

        std::fs::write(path, json).expect("Failed to write configuration");

        let mut configuration = self.configuration.lock().unwrap();
        let deep_link_setting_changed = (configuration.prusa_deep_link
            != new_configuration.prusa_deep_link
            && new_configuration.prusa_deep_link)
            || (configuration.cura_deep_link != new_configuration.cura_deep_link
                && new_configuration.cura_deep_link)
            || (configuration.bambu_deep_link != new_configuration.bambu_deep_link
                && new_configuration.bambu_deep_link)
            || (configuration.orca_deep_link != new_configuration.orca_deep_link
                && new_configuration.orca_deep_link);

        configuration.prusa_deep_link = new_configuration.prusa_deep_link;
        configuration.cura_deep_link = new_configuration.cura_deep_link;
        configuration.bambu_deep_link = new_configuration.bambu_deep_link;
        configuration.orca_deep_link = new_configuration.orca_deep_link;
        configuration.slicer = new_configuration.slicer.clone();
        configuration.thumbnail_color = new_configuration.thumbnail_color.clone();
        configuration.allow_importing_step = new_configuration.allow_importing_step;
        configuration.fallback_3mf_thumbnail = new_configuration.fallback_3mf_thumbnail;
        configuration.prefer_3mf_thumbnail = new_configuration.prefer_3mf_thumbnail;
        configuration.core_parallelism = new_configuration.core_parallelism;
        configuration.export_metadata = new_configuration.export_metadata;
        configuration.allow_importing_gcode = new_configuration.allow_importing_gcode;
        configuration.custom_slicer_path = new_configuration.custom_slicer_path.clone();
        configuration.elegoo_deep_link = new_configuration.elegoo_deep_link;
        configuration.prefer_gcode_thumbnail = new_configuration.prefer_gcode_thumbnail;

        deep_link_setting_changed
    }

    pub fn real_clone(&self) -> AppState {
        AppState {
            db: Arc::clone(&self.db),
            configuration: Mutex::new(self.get_configuration()),
            initial_state: self.initial_state.clone(),
            app_data_path: self.app_data_path.clone(),
            import_mutex: Arc::clone(&self.import_mutex),
            current_user: Arc::clone(&self.current_user),
        }
    }

    pub fn get_configuration(&self) -> Configuration {
        self.configuration.lock().unwrap().clone()
    }

    pub fn configure_deep_links(&self, app_handle: &AppHandle) {
        let config = self.get_configuration();

        if config.bambu_deep_link {
            let _ = app_handle.deep_link().register("bambustudio");
        }

        if config.cura_deep_link {
            let _ = app_handle.deep_link().register("cura");
        }

        if config.prusa_deep_link {
            let _ = app_handle.deep_link().register("prusaslicer");
        }

        if config.orca_deep_link {
            let _ = app_handle.deep_link().register("orcaslicer");
        }

        if config.elegoo_deep_link {
            let _ = app_handle.deep_link().register("elegooslicer");
        }

        let _ = app_handle.deep_link().register("meshorganiser");
    }
}

pub fn read_configuration(app_data_path: &str) -> Configuration {
    let path = PathBuf::from(app_data_path);
    let path = path.join("settings.json");

    if !path.exists() {
        return Configuration {
            data_path: String::from(app_data_path),
            ..Default::default()
        };
    }

    let json = std::fs::read_to_string(path).expect("Failed to read configuration");

    let stored_configuration: StoredConfiguration =
        serde_json::from_str(&json).expect("Failed to parse configuration");
    return stored_to_configuration(stored_configuration);
}
