use sqlx::Connection;
use tauri::Manager;
mod db;
mod configuration;
mod service;
mod error;
mod util;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let config = configuration::Configuration {
                    data_path : String::from(app.path().app_data_dir().expect("failed to get data_dir").to_str().unwrap()),
                    ..Default::default()
                };

                let db = db::db::setup_db(&config).await;
                
                app.manage(service::app_state::AppState 
                { 
                    db: db,
                    configuration: config,
                })
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
