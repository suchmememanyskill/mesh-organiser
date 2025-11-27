use std::path::PathBuf;

use service::{AppState, Configuration};
pub struct WebAppState {
    pub app_state: AppState,
    pub port: u16,
}

impl WebAppState {
    pub fn get_configuration(&self) -> Configuration {
        self.app_state.get_configuration()
    }

    pub fn get_model_dir(&self) -> PathBuf {
        self.app_state.get_model_dir()
    }

    pub fn get_image_dir(&self) -> PathBuf {
        self.app_state.get_image_dir()
    }

    pub fn get_signing_key_path(&self) -> PathBuf {
        PathBuf::from(&self.app_state.app_data_path).join("signing.key")
    }
}

impl Clone for WebAppState {
    fn clone(&self) -> Self {
        WebAppState {
            app_state: self.app_state.clone(),
            port: self.port,
        }
    }
}
