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
async fn edit_model(
    model_id : i64,
    model_name : &str,
    model_url : Option<&str>,
    model_description : Option<&str>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError>
{
    db::model::edit_model(model_id, model_name, model_url, model_description, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn get_labels(state: State<'_, AppState>) -> Result<Vec<db::label::Label>, ApplicationError> {
    let labels = db::label::get_labels(&state.db).await;

    Ok(labels)
}

#[tauri::command]
async fn get_configuration(state: State<'_, AppState>) -> Result<configuration::Configuration, ApplicationError> {
    Ok(state.configuration.clone())
}

#[tauri::command]
async fn delete_model(model_id: i64, state: State<'_, AppState>) -> Result<(), ApplicationError> {
    db::model::delete_model(model_id, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn add_label(
    label_name: &str,
    label_color: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::create_label(label_name, label_color, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn ungroup(
    group_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model_group::remove_group(group_id, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn edit_group(
    group_id: i64,
    group_name: &str,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model_group::edit_group(group_id, group_name, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn set_labels_on_model(
    label_ids: Vec<i64>,
    model_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::remove_labels_from_model(model_id, &state.db).await;
    db::label::set_labels_on_model(label_ids, model_id, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn open_in_slicer(
    model_ids : Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let models = db::model::get_models_by_id(model_ids, &state.db).await;

    state.configuration.slicer.open(models, &state)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    for entry in std::fs::read_dir(&std::env::temp_dir()).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("meshorganiser_open_action_") {
            println!("Removing temporary path {:?}", path);
            std::fs::remove_dir_all(&path).unwrap();
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let app_data_path = String::from(app.path()
                    .app_data_dir()
                    .expect("failed to get data_dir")
                    .to_str()
                    .unwrap());

                let config = configuration::Configuration {
                    data_path: String::from(&app_data_path),
                    model_path: String::from(&app_data_path),
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
        .invoke_handler(tauri::generate_handler![greet, add_model, get_models, get_labels, edit_model, delete_model, add_label, ungroup, edit_group, set_labels_on_model, open_in_slicer])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
