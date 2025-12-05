use std::{path::PathBuf, sync::{Arc, Mutex}};

use db::model::User;
use serde::{Deserialize, Serialize};
use service::{AppState, Configuration};
use tauri::AppHandle;
use tauri_plugin_deep_link::DeepLinkExt;

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountLinkEmit {
    pub base_url: String,
    pub user_name: String,
    pub link_token: String,
}

#[derive(Clone, Serialize)]
pub struct InitialState {
    pub deep_link_url: Option<String>,
    pub max_parallelism: usize,
    pub collapse_sidebar: bool,
    pub account_link: Option<AccountLinkEmit>,
}

pub struct TauriAppState {
    pub app_state: AppState,
    pub initial_state: InitialState,
    pub current_user: Arc<Mutex<User>>,
}

impl TauriAppState {
    pub fn get_configuration(&self) -> Configuration {
        self.app_state.get_configuration()
    }

    pub fn get_model_dir(&self) -> PathBuf {
        self.app_state.get_model_dir()
    }

    pub fn get_image_dir(&self) -> PathBuf {
        self.app_state.get_image_dir()
    }

    pub fn get_resources_dir(&self) -> PathBuf {
        self.app_state.get_resources_dir()
    }

    pub fn get_current_user(&self) -> User {
        let user = self.current_user.lock().unwrap();
        user.clone()
    }

    pub async fn set_current_user_by_id(&self, user_id: i64) -> Result<(), crate::error::ApplicationError> {
        let path = self.get_settings_path();
        let user = db::user_db::get_user_by_id(&self.app_state.db, user_id).await?;

        if user.is_none() {
            return Err(crate::error::ApplicationError::InternalError("User not found".into()));
        }

        let mut current_user = self.current_user.lock().unwrap();
        *current_user = user.unwrap();

        let mut configuration = self.app_state.configuration.lock().unwrap();
        configuration.last_user_id = user_id;

        let json = serde_json::to_string(&configuration.clone()).unwrap();
        std::fs::write(path, json).expect("Failed to write configuration");

        Ok(())
    }

    fn get_settings_path(&self) -> PathBuf {
        let mut path_buff = PathBuf::from(self.app_state.app_data_path.clone());
        path_buff.push("settings.json");
        path_buff
    }    

    pub fn write_configuration(&self, new_configuration: &Configuration) -> bool {
        let path = self.get_settings_path();
        let mut new_configuration = new_configuration.clone();
        new_configuration.last_user_id = self.get_current_user().id;
        let json = serde_json::to_string(&new_configuration).unwrap();

        std::fs::write(path, json).expect("Failed to write configuration");

        let mut configuration = self.app_state.configuration.lock().unwrap();
        let deep_link_setting_changed = (configuration.prusa_deep_link
            != new_configuration.prusa_deep_link
            && new_configuration.prusa_deep_link)
            || (configuration.cura_deep_link != new_configuration.cura_deep_link
                && new_configuration.cura_deep_link)
            || (configuration.bambu_deep_link != new_configuration.bambu_deep_link
                && new_configuration.bambu_deep_link)
            || (configuration.orca_deep_link != new_configuration.orca_deep_link
                && new_configuration.orca_deep_link);
        *configuration = new_configuration;

        deep_link_setting_changed
    }

    pub fn configure_deep_links(&self, app_handle: &AppHandle) {
        let config = self.app_state.get_configuration();

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