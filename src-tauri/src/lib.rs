use std::sync::{Arc, Mutex};

use error::ApplicationError;
use service::{
    app_state::AppState,
    model_service::{self, CreationResult},
};
use sqlx::Connection;
use tauri::{Manager, State};
mod configuration;
mod db;
mod error;
mod service;
mod util;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn add_model(
    path: &str,
    state: State<'_, AppState>,
) -> Result<CreationResult, ApplicationError> {
    let path_clone = String::from(path);
    let state_clone = state.real_clone();

    let result = tauri::async_runtime::spawn_blocking(move || model_service::import_path(&path_clone, &state_clone)).await.unwrap()?;

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let config = configuration::Configuration {
                    data_path: String::from(
                        app.path()
                            .app_data_dir()
                            .expect("failed to get data_dir")
                            .to_str()
                            .unwrap(),
                    ),
                    ..Default::default()
                };

                let db = db::db::setup_db(&config).await;

                app.manage(AppState {
                    db: Arc::new(db),
                    configuration: config,
                })
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, add_model])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
