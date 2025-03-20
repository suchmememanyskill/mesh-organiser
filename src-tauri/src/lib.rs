use std::sync::{Arc, Mutex};

use error::ApplicationError;
use service::{
    app_state::AppState,
    model_service::{self, CreationResult},
};
use sqlx::Connection;
use tauri::{Manager, State, AppHandle};
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
    app_handle: AppHandle,
) -> Result<CreationResult, ApplicationError> {
    let path_clone = String::from(path);
    let state_clone = state.real_clone();

    let result = tauri::async_runtime::spawn_blocking(move || model_service::import_path(&path_clone, &state_clone)).await.unwrap()?;

    service::thumbnail_service::generate_all_thumbnails(&state, &app_handle, false).await?;

    Ok(result)
}

#[tauri::command]
async fn get_models(state: State<'_, AppState>) -> Result<Vec<db::model::Model>, ApplicationError> {
    let models = db::model::get_models(&state.db).await;

    Ok(models)
}

#[tauri::command]
async fn get_labels(state: State<'_, AppState>) -> Result<Vec<db::label::Label>, ApplicationError> {
    let labels = db::label::get_labels(&state.db).await;

    Ok(labels)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
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
        .invoke_handler(tauri::generate_handler![greet, add_model, get_models, get_labels])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
