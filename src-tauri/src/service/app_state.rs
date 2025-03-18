use crate::db;
use crate::configuration;
use std::path::PathBuf;

pub struct AppState {
    pub db: db::db::Db,
    pub configuration: configuration::Configuration
}

impl AppState 
{
    pub fn get_model_dir(&self) -> String
    {
        let mut path_buff = PathBuf::from(self.configuration.data_path.clone());
        path_buff.push("models");
        
        if !path_buff.exists()
        {
            std::fs::create_dir_all(path_buff.clone()).expect("Failed to create model directory");
        }

        String::from(path_buff.to_str().unwrap())
    }
}
