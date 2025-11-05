use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use arboard::Clipboard;
use base64::prelude::*;
use configuration::Configuration;
use db::{
    model::ModelFlags,
    model::{Resource, ResourceFlags},
};
use error::ApplicationError;
use serde::Serialize;
use service::{
    app_state::{AppState, InitialState, read_configuration},
    download_file_service, import_service,
    slicer_service::Slicer,
};
use std::fs::File;
use std::io::prelude::*;
use strum::IntoEnumIterator;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri::{
    WebviewUrl, WebviewWindowBuilder,
    async_runtime::block_on,
    menu::{MenuBuilder, SubmenuBuilder},
    webview::{DownloadEvent, PageLoadEvent},
};
use urlencoding::decode;

use crate::service::import_state::{ImportState, ImportStatus};
mod configuration;
mod error;
mod service;
mod util;
mod web_server;

#[derive(Serialize, Clone)]
struct DeepLinkEmit {
    download_url: String,
    source_url: Option<String>,
}

#[tauri::command]
async fn add_model(
    path: &str,
    recursive: bool,
    delete_imported: bool,
    origin_url: Option<String>,
    open_in_slicer: bool,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ImportState, ApplicationError> {
    let path_clone = String::from(path);
    let state_clone = state.real_clone();
    let handle_clone = app_handle.clone();
    let mut import_state = ImportState::new(origin_url, recursive, delete_imported);

    import_state = tauri::async_runtime::spawn_blocking(move || {
        let lock = state_clone.import_mutex.lock().unwrap();
        import_service::import_path(&path_clone, &state_clone, &handle_clone, &mut import_state)?;

        Result::<ImportState, ApplicationError>::Ok(import_state)
    })
    .await
    .unwrap()?;

    let model_ids: Vec<i64> = import_state
        .imported_models
        .iter()
        .flat_map(|f| f.model_ids.clone())
        .collect();
    let models = db::model::get_models_by_id(model_ids, &state.db).await;
    service::thumbnail_service::generate_thumbnails(
        &models,
        &state,
        &app_handle,
        false,
        &mut import_state,
    )
    .await?;

    if open_in_slicer && models.len() > 0 {
        if let Some(slicer) = &state.get_configuration().slicer {
            slicer.open(models, &state)?;
        }
    }

    import_state.status = ImportStatus::Finished;
    Ok(import_state)
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
    model_flags: ModelFlags,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model::edit_model(
        model_id,
        model_name,
        model_url,
        model_description,
        model_flags,
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
    let model = db::model::get_models_by_id(vec![model_id], &state.db).await;

    if model.len() <= 0 {
        return Err(ApplicationError::InternalError(String::from(
            "Failed to find model to delete",
        )));
    }

    let model = &model[0];

    db::model::delete_model(model_id, &state.db).await;

    let model_path =
        PathBuf::from(state.get_model_dir()).join(format!("{}.{}", model.sha256, model.filetype));
    let image_path = PathBuf::from(state.get_image_dir()).join(format!("{}.png", model.sha256));

    if model_path.exists() {
        std::fs::remove_file(model_path)?;
    }

    if image_path.exists() {
        std::fs::remove_file(image_path)?;
    }

    Ok(())
}

#[tauri::command]
async fn add_label(
    label_name: &str,
    label_color: i64,
    state: State<'_, AppState>,
) -> Result<i64, ApplicationError> {
    let id = db::label::create_label(label_name, label_color, &state.db).await;

    Ok(id)
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
    group_resource_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::model_group::edit_group(group_id, group_name, group_resource_id, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn set_labels_on_model(
    label_ids: Vec<i64>,
    model_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::remove_labels_from_model(model_id, &state.db).await;
    db::label::add_labels_on_model(label_ids, model_id, &state.db).await;

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

#[tauri::command]
async fn get_model_as_base64(
    model_id: i64,
    state: State<'_, AppState>,
) -> Result<String, ApplicationError> {
    let model = db::model::get_models_by_id(vec![model_id], &state.db).await;

    if model.len() <= 0 {
        return Err(ApplicationError::InternalError(String::from(
            "Failed to find model to delete",
        )));
    }

    let model = &model[0];

    let bytes = service::export_service::get_bytes_from_model(model, &state).unwrap();
    let base64 = BASE64_STANDARD.encode(bytes);

    Ok(base64)
}

#[tauri::command]
async fn get_model_bytes(
    model_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, ApplicationError> {
    let model = db::model::get_models_by_id(vec![model_id], &state.db).await;

    if model.len() <= 0 {
        return Err(ApplicationError::InternalError(String::from(
            "Failed to find model to delete",
        )));
    }

    let model = &model[0];


    let bytes = service::export_service::get_bytes_from_model(model, &state).unwrap();

    Ok(bytes)
}

#[derive(Serialize, Clone)]
struct DownloadFinishedEvent {
    path: String,
    url: String,
}

struct Site {
    name: &'static str,
    url: &'static str,
}

#[tauri::command]
async fn new_window_with_url(url: &str, app_handle: AppHandle) -> Result<(), ApplicationError> {
    let cloned_handle = app_handle.clone();
    if let Some(window) = app_handle.webview_windows().get("secondary") {
        window.set_focus()?;
        window.navigate(url.parse().unwrap())?;
        window.set_title("Browse models")?;
        return Ok(());
    }

    let sites: Vec<Site> = vec![
        Site {
            name: "Thingiverse",
            url: "https://www.thingiverse.com/",
        },
        Site {
            name: "MyMiniFactory",
            url: "https://www.myminifactory.com/search#/?{\"designType\":\"free-only\"}",
        },
        Site {
            name: "Printables",
            url: "https://www.printables.com",
        },
        Site {
            name: "Makerworld",
            url: "https://www.makerworld.com",
        },
    ];

    println!("Opening new window with URL: {}", url);

    let mut submenu = SubmenuBuilder::new(&app_handle, "Sites");

    for site in sites.iter() {
        submenu = submenu.text(site.url, site.name);
    }

    let menu = MenuBuilder::new(&app_handle)
        .text("back", "← Back")
        .separator()
        .text("reload", "⟳ Reload")
        .separator()
        .text("forward", "→ Forward")
        .separator()
        .text("copy_url", "Copy URL")
        .separator()
        .build()?;

    menu.append(&submenu.build()?)?;

    WebviewWindowBuilder::new(
        &app_handle,
        "secondary",
        WebviewUrl::External(url.parse().unwrap()),
    )
    .title("Browse models")
    .center()
    .inner_size(1280f64, 720f64)
    .menu(menu)
    .on_menu_event(|f, event| {
        let webviews = f.webviews();
        let webview = webviews.first().unwrap();
        let id = event.id().0.as_str();

        match id {
            "back" => {
                let _ = webview.eval("window.history.back()");
            }
            "forward" => {
                let _ = webview.eval("window.history.forward()");
            }
            "reload" => {
                let _ = webview.eval("window.location.reload()");
            }
            "copy_url" => {
                let url = webview.url().unwrap();
                let mut clipboard = Clipboard::new().unwrap();
                let _ = clipboard.set_text(url.to_string());
            }
            _ => {
                let _ = webview.navigate(id.parse().unwrap());
            }
        }
    })
    .on_navigation(move |f| {
        let url = f.to_string();
        println!("Navigated to: {}", url);

        if let Some(deep_link) = extract_deep_link(&url) {
            println!("Extracted deep link: {:?}", &deep_link);

            let window = cloned_handle.get_webview_window("secondary");

            if let Some(window) = window {
                if let Ok(source_url) = window.url() {
                    let _ = cloned_handle.emit(
                        "deep-link",
                        DeepLinkEmit {
                            download_url: deep_link,
                            source_url: Some(source_url.to_string()),
                        },
                    );
                }
            }

            return false;
        }

        true
    })
    .on_page_load(|f, e| {
        if e.event() == PageLoadEvent::Finished {
            let _ = f.eval(include_str!("./inject.js"));
        }
    })
    .on_download(|f, event| {
        if let DownloadEvent::Requested { url, destination } = &event {
            println!("Download started: {:?}", url);
            let _ = f.app_handle().emit("download-started", url).unwrap();
            let _ = f.window().set_title("Downloading model...");
        }

        if let DownloadEvent::Finished { url, path, success } = event {
            if path.is_some() && success {
                let path = path.unwrap();
                let handle = f.app_handle();

                println!("Download finished: {:?}", path);
                let _ = handle
                    .emit(
                        "download-finished",
                        DownloadFinishedEvent {
                            path: String::from(path.to_str().unwrap()),
                            url: String::from(f.url().unwrap()),
                        },
                    )
                    .unwrap();

                let _ = f.window().set_title("Download complete");
            }
        }

        true
    })
    .build()?;

    Ok(())
}

#[tauri::command]
async fn add_childs_to_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::add_childs_to_label(parent_label_id, child_label_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn remove_childs_from_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::remove_childs_from_label(parent_label_id, child_label_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn set_childs_on_label(
    parent_label_id: i64,
    child_label_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label::remove_all_childs_from_label(parent_label_id, &state.db).await;
    db::label::add_childs_to_label(parent_label_id, child_label_ids, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn get_resources(state: State<'_, AppState>) -> Result<Vec<Resource>, ApplicationError> {
    let resources = db::resource::get_resources(&state.db).await;

    Ok(resources)
}

#[tauri::command]
async fn add_resource(
    resource_name: &str,
    state: State<'_, AppState>,
) -> Result<i64, ApplicationError> {
    let id = db::resource::add_resource(resource_name, &state.db).await;

    Ok(id)
}

#[tauri::command]
async fn edit_resource(
    resource_id: i64,
    resource_name: &str,
    resource_flags: ResourceFlags,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::resource::edit_resource(resource_id, resource_name, resource_flags, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn remove_resource(
    resource_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let resource = db::resource::get_resource_by_id(resource_id, &state.db).await;

    if resource.is_none() {
        return Err(ApplicationError::InternalError(String::from(
            "Resource not found",
        )));
    }

    let resource = resource.unwrap();

    service::resource_service::delete_resource_folder(&resource, &state).await?;
    db::resource::delete_resource(resource.id, &state.db).await;

    Ok(())
}

#[tauri::command]
async fn open_resource_folder(
    resource_id: i64,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    let resource = db::resource::get_resource_by_id(resource_id, &state.db).await;

    if resource.is_none() {
        return Err(ApplicationError::InternalError(String::from(
            "Resource not found",
        )));
    }

    let resource = resource.unwrap();

    service::resource_service::open_resource_folder(&resource, &state).await?;
    Ok(())
}

#[tauri::command]
async fn set_keywords_on_label(
    label_id: i64,
    keywords: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), ApplicationError> {
    db::label_keywords::set_keywords_for_label(&state.db, label_id, keywords).await;

    Ok(())
}

#[tauri::command]
async fn get_keywords_for_label(
    label_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<db::label_keywords::LabelKeyword>, ApplicationError> {
    let keywords = db::label_keywords::get_keywords_for_label(&state.db, label_id).await;

    Ok(keywords)
}

fn extract_deep_link(data: &str) -> Option<String> {
    let possible_starts = vec![
        "bambustudio://open/?file=",
        "cura://open/?file=",
        "prusaslicer://open/?file=",
        "orcaslicer://open/?file=",
        "elegooslicer://open/?file=",
        "meshorganiser://open/?file=",
        "bambustudio://open?file=",
        "cura://open?file=",
        "prusaslicer://open?file=",
        "orcaslicer://open?file=",
        "elegooslicer://open?file=",
        "meshorganiser://open?file=",
    ];

    for start in possible_starts {
        if data.starts_with(start) {
            let encoded = data[start.len()..].to_string();

            if data.starts_with("elegooslicer") {
                return Some(encoded);
            }

            let decode = decode(&encoded).unwrap();

            return Some(String::from(decode));
        }
    }

    None
}

fn remove_temp_paths() -> Result<(), ApplicationError> {
    let threshold = std::time::Duration::from_secs(5 * 60);
    let now = std::time::SystemTime::now();
    for entry in std::fs::read_dir(&std::env::temp_dir())? {
        let entry = entry?;
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
                        std::fs::remove_dir_all(&path)?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    thread::spawn(move || {
        let _ = remove_temp_paths();
    });

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            println!("a new app instance was opened with {argv:?} and the deep link event was already triggered");

            if argv.len() == 2
            {
                let deep_link = extract_deep_link(&argv[1]);

                if let Some(deep_link) = deep_link
                {
                    println!("Emitting deep link {:?}", deep_link);
                    _app.emit("deep-link", DeepLinkEmit { download_url: deep_link, source_url: None } ).unwrap();
                }
                else
                {
                    println!("Failed to extract deep link {:?}", &argv[1]);
                }
            }
            else
            {
                let window = _app.get_webview_window("main");

                if let Some(window) = window {
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
                else
                {
                    println!("Failed to get window to focus");
                }
            }
          }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "main" {
                    for window in window.app_handle().webview_windows() {
                        let _ = window.1.close();
                    }
                }
            }
        })
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let app_data_path = String::from(
                    app.path()
                        .app_data_dir()
                        .expect("failed to get data_dir")
                        .to_str()
                        .unwrap(),
                );

                let app_data_path_clone = String::from(&app_data_path);

                std::panic::set_hook(Box::new(move |info| {
                    let loc = PathBuf::from(&app_data_path_clone).join("crash.log");

                    let error_message = match info.payload().downcast_ref::<&'static str>()
                    {
                        Some(s) => *s,
                        None => match info.payload().downcast_ref::<String>() {
                            Some(s) => &s[..],
                            None => "Box<Any>",
                        },
                    };

                    if let Ok(mut file) = File::create(loc)
                    {
                        let _ = writeln!(file, "Panic occurred: {error_message}\n{info}\n{:#?}", info);
                    }

                    println!("Panic occurred: {error_message}\n{:?}", info);
                }));

                let config = read_configuration(&app_data_path);

                // Please do not touch this error. I will fix it later.
                let db = db::db::setup_db(&config, &app_data_path).await;

                let mut initial_state = InitialState {
                    deep_link_url: None,
                    max_parallelism: std::thread::available_parallelism()
                        .unwrap_or(std::num::NonZeroUsize::new(6).unwrap())
                        .get(),
                    collapse_sidebar: config.collapse_sidebar,
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
                    import_mutex: Arc::new(Mutex::new(())),
                };

                state.configure_deep_links(&app.handle());

                app.manage(state);

                let handle = app.handle().clone();

                thread::spawn(move || {
                    web_server::init(handle).unwrap();
                });               
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
            new_window_with_url,
            get_model_as_base64,
            add_childs_to_label,
            remove_childs_from_label,
            set_childs_on_label,
            get_resources,
            add_resource,
            edit_resource,
            remove_resource,
            open_resource_folder,
            set_keywords_on_label,
            get_keywords_for_label,
            get_model_bytes,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, e| {
        if let tauri::RunEvent::ExitRequested { .. } = e {
            // Close sqlite db
            tauri::async_runtime::block_on(async move {
                let app_state = _app_handle.state::<AppState>();
                app_state.db.close().await;
            });
        }
    });
}
