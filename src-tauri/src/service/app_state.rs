use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_deep_link::DeepLinkExt;

use crate::configuration;
use crate::db;
use configuration::{stored_to_configuration, Configuration, StoredConfiguration};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Serialize)]
pub struct InitialState {
    pub deep_link_url: Option<String>,
}

pub struct AppState {
    pub db: Arc<db::db::Db>,
    // TODO: Put behind a mutex so i can change the configuration during runtime
    pub configuration: Mutex<Configuration>,
    pub initial_state: InitialState,
    pub app_data_path: String,
}

impl AppState {
    pub fn get_model_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.get_configuration().data_path.clone());
        path_buff.push("models");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create model directory");
        }

        String::from(path_buff.to_str().unwrap())
    }

    pub fn get_image_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.app_data_path.clone());
        path_buff.push("images");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create image directory");
        }

        String::from(path_buff.to_str().unwrap())
    }

    pub fn write_configuration(&self, new_configuration: &Configuration) -> bool {
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

        deep_link_setting_changed
    }

    pub fn real_clone(&self) -> AppState {
        AppState {
            db: Arc::clone(&self.db),
            configuration: Mutex::new(self.get_configuration()),
            initial_state: self.initial_state.clone(),
            app_data_path: self.app_data_path.clone(),
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
