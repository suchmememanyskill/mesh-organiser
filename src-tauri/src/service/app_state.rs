use serde::Serialize;

use crate::configuration;
use crate::db;
use std::path::PathBuf;
use std::sync::Arc;
use configuration::Configuration;

#[derive(Clone, Serialize)]
pub struct InitialState 
{
    pub deep_link_url : Option<String>,
}

pub struct AppState {
    pub db: Arc<db::db::Db>,
    // TODO: Put behind a mutex so i can change the configuration during runtime
    pub configuration: configuration::Configuration,
    pub initial_state: InitialState,
    pub app_data_path: String,
}

impl AppState {
    pub fn get_model_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.configuration.data_path.clone());
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

    pub fn write_configuration(&self, configuration: &Configuration) {
        let path = PathBuf::from(self.app_data_path.clone());
        let path = path.join("settings.json");

        let json = serde_json::to_string(&configuration).unwrap();

        std::fs::write(path, json).expect("Failed to write configuration");
    }

    pub fn real_clone(&self) -> AppState {
        AppState {
            db: Arc::clone(&self.db),
            configuration: self.configuration.clone(),
            initial_state: self.initial_state.clone(),
            app_data_path: self.app_data_path.clone(),
        }
    }
}


pub fn read_configuration(app_data_path: &str) -> Configuration {
    let path = PathBuf::from(app_data_path);
    let path = path.join("settings.json");

    if !path.exists() {
        return Configuration {
            data_path: String::from(app_data_path),
            ..Default::default()
        }
    }

    let json = std::fs::read_to_string(path).expect("Failed to read configuration");

    serde_json::from_str(&json).expect("Failed to parse configuration")
}