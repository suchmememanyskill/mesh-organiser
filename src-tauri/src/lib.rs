use std::sync::{Arc, Mutex};

use configuration::Configuration;
use error::ApplicationError;
use serde::Serialize;
use service::{
    app_state::{read_configuration, AppState, InitialState},
    download_file_service,
    model_service::{self, CreationResult},
    slicer_service::Slicer,
};
use strum::IntoEnumIterator;
use tauri::async_runtime::block_on;
use tauri::{AppHandle, Emitter, Manager, State};
use urlencoding::decode;
mod configuration;
mod db;
mod error;
mod service;
mod util;

#[tauri::command]
async fn add_model(
    path: &str,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<CreationResult, ApplicationError> {
    let path_clone = String::from(path);
    let state_clone = state.real_clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let result = model_service::import_path(&path_clone, &state_clone, &app_handle)?;
        let models = db::model::get_models_by_id_sync(result.model_ids.clone(), &state_clone.db);
        block_on(service::thumbnail_service::generate_thumbnails(
            models,
            &state_clone,
            &app_handle,
            false,
        ))?;

        Result::<CreationResult, ApplicationError>::Ok(result)
    })
    .await
    .unwrap()?;

    Ok(result)
}

#[tauri::command]
async fn get_models(state: State<'_, AppState>) -> Result<Vec<db::model::Model>, ApplicationError> {
    let models = db::model::get_models(&state.db).await;

    Ok(models)
}

#[tauri::command]
async fn edit_model(
    model_id: i64,
    model_name: &str,
    model_url: Option<&str>,
    model_description: Option<&str>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model::edit_model(
        model_id,
        model_name,
        model_url,
        model_description,
        &state.db,
    )
    .await;

    Ok(())
}

#[tauri::command]
async fn get_labels(state: State<'_, AppState>) -> Result<Vec<db::label::Label>, ApplicationError> {
    let labels = db::label::get_labels(&state.db).await;

    Ok(labels)
}

#[tauri::command]
async fn get_configuration(
    state: State<'_, AppState>,
) -> Result<configuration::Configuration, ApplicationError> {
    Ok(state.get_configuration())
}

#[tauri::command]
async fn set_configuration(
    configuration: Configuration,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), ApplicationError> {
    let deep_link_state_changed = state.write_configuration(&configuration);

    if deep_link_state_changed {
        state.configure_deep_links(&app_handle);
    }

    Ok(())
}

#[derive(Serialize)]
pub struct SlicerEntry {
    slicer: Slicer,
    installed: bool,
}

#[tauri::command]
async fn get_slicers() -> Result<Vec<SlicerEntry>, ApplicationError> {
    Slicer::iter()
        .map(|f| {
            let installed = f.is_installed();

            Ok(SlicerEntry {
                slicer: f,
                installed: installed,
            })
        })
        .collect()
}

#[tauri::command]
async fn update_images(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    overwrite: bool,
) -> Result<(), ApplicationError> {
    service::thumbnail_service::generate_all_thumbnails(&state, &app_handle, overwrite).await?;

    Ok(())
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
async fn add_group(group_name: &str, state: State<'_, AppState>) -> Result<i64, ApplicationError> {
    let id = db::model_group::add_empty_group(group_name, &state.db).await;

    Ok(id)
}

#[tauri::command]
async fn add_models_to_group(
    group_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model_group::set_group_id_on_models(Some(group_id), model_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn remove_models_from_group(
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model_group::set_group_id_on_models(None, model_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn ungroup(group_id: i64, state: State<'_, AppState>) -> Result<(), ApplicationError> {
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
async fn set_label_on_models(
    label_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::remove_label_from_models(label_id, model_ids.clone(), &state.db).await;
    db::label::add_label_on_models(label_id, model_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn remove_label_from_models(
    label_id: i64,
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::remove_label_from_models(label_id, model_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn edit_label(
    label_id: i64,
    label_name: &str,
    label_color: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::edit_label(label_id, label_name, label_color, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn delete_label(label_id: i64, state: State<'_, AppState>) -> Result<(), ApplicationError> {
    db::label::delete_label(label_id, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn open_in_slicer(
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let models = db::model::get_models_by_id(model_ids, &state.db).await;

    if let Some(slicer) = &state.get_configuration().slicer {
        slicer.open(models, &state)?;
    }

    Ok(())
}

#[tauri::command]
async fn get_initial_state(state: State<'_, AppState>) -> Result<InitialState, ApplicationError> {
    Ok(state.initial_state.clone())
}

#[tauri::command]
async fn download_file(
    url: &str,
) -> Result<download_file_service::DownloadResult, ApplicationError> {
    let response = download_file_service::download_file(url).await?;

    Ok(response)
}

#[tauri::command]
async fn open_in_folder(
    model_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let models = db::model::get_models_by_id(model_ids, &state.db).await;

    let (temp_dir, _) =
        service::export_service::export_to_temp_folder(models, &state, false, "export").unwrap();

    crate::util::open_folder_in_explorer(temp_dir.to_str().unwrap());

    Ok(())
}

#[tauri::command]
async fn remove_dead_groups(state: State<'_, AppState>) -> Result<(), ApplicationError> {
    db::model_group::remove_dead_groups(&state.db).await;

    Ok(())
}

#[tauri::command]
async fn compute_model_folder_size(state: State<'_, AppState>) -> Result<u64, ApplicationError> {
    let size = util::get_folder_size(&state.get_model_dir());

    Ok(size)
}

fn extract_deep_link(data: &str) -> Option<String> {
    let possible_starts = vec![
        "bambustudio://open/?file=",
        "cura://open/?file=",
        "prusaslicer://open/?file=",
        "orcaslicer://open/?file=",
        "meshorganiser://open/?file=",
    ];

    for start in possible_starts {
        if data.starts_with(start) {
            let encoded = data[start.len()..].to_string();
            let decode = decode(&encoded).unwrap();

            return Some(String::from(decode));
        }
    }

    None
}

fn remove_temp_paths() {
    let threshold = std::time::Duration::from_secs(5 * 60);
    let now = std::time::SystemTime::now();
    for entry in std::fs::read_dir(&std::env::temp_dir()).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("meshorganiser_")
        {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if now
                        .duration_since(modified)
                        .unwrap_or(std::time::Duration::ZERO)
                        >= threshold
                    {
                        println!("Removing temporary path {:?}", path);
                        std::fs::remove_dir_all(&path).unwrap();
                    }
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    remove_temp_paths();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            println!("a new app instance was opened with {argv:?} and the deep link event was already triggered");

            if argv.len() == 2
            {
                let deep_link = extract_deep_link(&argv[1]);

                if let Some(deep_link) = deep_link
                {
                    _app.emit("deep-link", deep_link).unwrap();
                }
            }
            
          }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let app_data_path = String::from(
                    app.path()
                        .app_data_dir()
                        .expect("failed to get data_dir")
                        .to_str()
                        .unwrap(),
                );

                let config = read_configuration(&app_data_path);

                let db = db::db::setup_db(&config).await;

                let mut initial_state = InitialState {
                    deep_link_url: None,
                    max_parallelism: std::thread::available_parallelism()
                        .unwrap_or(std::num::NonZeroUsize::new(6).unwrap())
                        .get(),
                };

                let argv = std::env::args();

                if argv.len() == 2
                {
                    let arg = argv.skip(1).next().unwrap();
                    let deep_link = extract_deep_link(&arg);
    
                    if let Some(deep_link) = deep_link
                    {
                        initial_state.deep_link_url = Some(deep_link);
                    }
                }

                let state = AppState {
                    db: Arc::new(db),
                    configuration: Mutex::new(config),
                    initial_state: initial_state,
                    app_data_path: app_data_path,
                };

                state.configure_deep_links(&app.handle());

                app.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_model,
            get_models,
            get_labels,
            edit_model,
            delete_model,
            add_label,
            ungroup,
            edit_group,
            set_labels_on_model,
            open_in_slicer,
            get_initial_state,
            download_file,
            open_in_folder,
            set_label_on_models,
            remove_label_from_models,
            add_group,
            add_models_to_group,
            remove_models_from_group,
            remove_dead_groups,
            edit_label,
            delete_label,
            update_images,
            get_slicers,
            set_configuration,
            get_configuration,
            compute_model_folder_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
