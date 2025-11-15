use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use db::user_db;
use arboard::Clipboard;
use base64::prelude::*;

use db::{
    label_db, model::{ModelFlags, Resource, ResourceFlags, User}, model_db
};
use error::ApplicationError;
use serde::Serialize;
use service::{
    download_file_service, import_service,
    slicer_service::Slicer,
};
use std::fs::File;
use std::io::prelude::*;
use strum::IntoEnumIterator;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri::{
    WebviewUrl, WebviewWindowBuilder,
    menu::{MenuBuilder, SubmenuBuilder},
    webview::{DownloadEvent, PageLoadEvent},
};
use urlencoding::decode;
use service::import_state::ImportState;
use service::Configuration;
use crate::tauri_app_state::TauriAppState;
use crate::tauri_app_state::InitialState;
use service::StoredConfiguration;
use service::stored_to_configuration;
use service::AppState;

mod tauri_app_state;
mod error;
mod api;
mod tauri_import_state;
mod tauri_thumbnail_service;

#[derive(Serialize, Clone)]
struct DeepLinkEmit {
    download_url: String,
    source_url: Option<String>,
}


#[tauri::command]
async fn get_configuration(
    state: State<'_, TauriAppState>,
) -> Result<Configuration, ApplicationError> {
    Ok(state.get_configuration())
}

#[tauri::command]
async fn set_configuration(
    configuration: Configuration,
    state: State<'_, TauriAppState>,
    app_handle: AppHandle,
) -> Result<(), ApplicationError> {
    let mut configuration = configuration;
    let deep_link_state_changed = state.write_configuration(&mut configuration);

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
    state: State<'_, TauriAppState>,
    app_handle: AppHandle,
    overwrite: bool,
) -> Result<(), ApplicationError> {
    let _lock = state.app_state.import_mutex.lock().await;
    crate::tauri_thumbnail_service::generate_all_thumbnails(&state, &app_handle, overwrite).await?;

    Ok(())
}

#[tauri::command]
async fn open_in_slicer(
    model_ids: Vec<i64>,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    let models = model_db::get_models_via_ids(&state.app_state.db, &state.get_current_user(), model_ids).await?;

    if let Some(slicer) = &state.get_configuration().slicer {
        slicer.open(models, &state.app_state).await?;
    }

    Ok(())
}

#[tauri::command]
async fn get_initial_state(state: State<'_, TauriAppState>) -> Result<InitialState, ApplicationError> {
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
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    let models = model_db::get_models_via_ids(&state.app_state.db, &state.get_current_user(), model_ids).await?;

    let (temp_dir, _) =
        service::export_service::export_to_temp_folder(models, &state.app_state, false, "export").await?;

    service::open_folder_in_explorer(&temp_dir);

    Ok(())
}


#[tauri::command]
async fn compute_model_folder_size(state: State<'_, TauriAppState>) -> Result<u64, ApplicationError> {
    let size = service::get_folder_size(&state.get_model_dir());

    Ok(size)
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
        if let DownloadEvent::Requested { url, destination: _ } = &event {
            println!("Download started: {:?}", url);
            let _ = f.app_handle().emit("download-started", url).unwrap();
            let _ = f.window().set_title("Downloading model...");
        }

        if let DownloadEvent::Finished { url: _, path, success } = event {
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

pub fn read_configuration(app_data_path: &str) -> Configuration {
    let path = PathBuf::from(app_data_path);
    let path = path.join("settings.json");

    if !path.exists() {
        return Configuration {
            data_path: String::from(app_data_path),
            ..Default::default()
        };
    }

    let json = std::fs::read_to_string(path).expect("Failed to read configuration");

    let stored_configuration: StoredConfiguration =
        serde_json::from_str(&json).expect("Failed to parse configuration");
    return stored_to_configuration(stored_configuration);
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

                let sqlite_path = PathBuf::from(&app_data_path).join("db.sqlite");
                let sqlite_backup_dir = PathBuf::from(&app_data_path).join("backups");
                let db = db::db_context::setup_db(&sqlite_path, &sqlite_backup_dir).await;

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

                let user = user_db::get_user_by_id(&db, config.last_user_id).await.ok().flatten().unwrap_or(User::default());

                let state = TauriAppState {
                    app_state: AppState {
                        db: Arc::new(db),
                        configuration: Mutex::new(config),
                        import_mutex: Arc::new(tokio::sync::Mutex::new(())),
                        app_data_path: app_data_path,
                    },
                    initial_state: initial_state,
                    current_user: Arc::new(Mutex::new(user)),
                };

                state.configure_deep_links(&app.handle());

                app.manage(state);

                /*
                let handle = app.handle().clone();

                thread::spawn(move || {
                    web_server::init(handle).unwrap();
                });
                */
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::add_model,
            api::get_models,
            api::get_labels,
            api::edit_model,
            api::delete_model,
            api::add_label,
            api::ungroup,
            api::edit_group,
            api::set_labels_on_model,
            open_in_slicer,
            get_initial_state,
            download_file,
            open_in_folder,
            api::set_label_on_models,
            api::remove_label_from_models,
            api::add_group,
            api::add_models_to_group,
            api::remove_models_from_group,
            api::remove_dead_groups,
            api::edit_label,
            api::delete_label,
            update_images,
            get_slicers,
            set_configuration,
            get_configuration,
            compute_model_folder_size,
            new_window_with_url,
            api::add_childs_to_label,
            api::remove_childs_from_label,
            api::set_childs_on_label,
            api::get_resources,
            api::add_resource,
            api::edit_resource,
            api::remove_resource,
            api::open_resource_folder,
            api::set_keywords_on_label,
            api::get_keywords_for_label,
            api::get_model_bytes,
            api::get_blob_bytes,
            api::get_current_user,
            api::set_current_user,
            api::get_users,
            api::add_user,
            api::edit_user,
            api::delete_user,
            api::get_groups,
            api::set_resource_on_group,
            api::get_group_count,
            api::get_model_count,
            api::get_groups_for_resource,
            api::get_model_disk_space_usage,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, e| {
        if let tauri::RunEvent::ExitRequested { .. } = e {
            // Close sqlite db
            tauri::async_runtime::block_on(async move {
                let app_state = _app_handle.state::<TauriAppState>();
                app_state.app_state.db.close().await;
            });
        }
    });
}
