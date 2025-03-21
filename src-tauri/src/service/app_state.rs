use crate::configuration;
use crate::db;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<db::db::Db>,
    pub configuration: configuration::Configuration,
}

impl AppState {
    pub fn get_model_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.configuration.model_path.clone());
        path_buff.push("models");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create model directory");
        }

        String::from(path_buff.to_str().unwrap())
    }

    pub fn get_image_dir(&self) -> String {
        let mut path_buff = PathBuf::from(self.configuration.data_path.clone());
        path_buff.push("images");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create image directory");
        }

        String::from(path_buff.to_str().unwrap())
    }

    pub fn real_clone(&self) -> AppState {
        AppState {
            db: Arc::clone(&self.db),
            configuration: self.configuration.clone(),
        }
    }
}
