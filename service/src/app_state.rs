use db::db_context::DbContext;

use crate::configuration;
use configuration::Configuration;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

pub struct AppState {
    pub db: Arc<DbContext>,
    pub configuration: Mutex<Configuration>,
    pub import_mutex: Arc<tokio::sync::Mutex<()>>,
    pub app_data_path: String,
}

impl AppState {
    pub fn get_model_dir(&self) -> PathBuf {
        let mut path_buff = PathBuf::from(self.get_configuration().data_path.clone());
        path_buff.push("models");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create model directory");
        }

        path_buff
    }

    pub fn get_image_dir(&self) -> PathBuf {
        let mut path_buff = PathBuf::from(self.app_data_path.clone());
        path_buff.push("images");

        if !path_buff.exists() {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create image directory");
        }

        path_buff
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

    pub fn get_configuration(&self) -> Configuration {
        self.configuration.lock().unwrap().clone()
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            configuration: Mutex::new(self.get_configuration()),
            app_data_path: self.app_data_path.clone(),
            import_mutex: Arc::clone(&self.import_mutex),
        }
    }
}