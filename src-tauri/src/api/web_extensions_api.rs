use std::{char::MAX, panic, path::{self, PathBuf}, sync::Arc};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use service::{export_service::{ensure_unique_file_full_filename, get_temp_dir}, import_service::{self, DirectoryScanModel, is_any_supported_extension}, import_state::{ImportState, ImportStatus}};
use tauri::{AppHandle, State, http::header::CONTENT_DISPOSITION, ipc::Response};
use tauri_plugin_http::reqwest::{self, cookie::Jar};
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};

use crate::{error::ApplicationError, tauri_app_state::TauriAppState, tauri_import_state};

async fn download_file(url : &str, dir : &PathBuf) -> Result<PathBuf, ApplicationError> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(ApplicationError::InternalError("Got a non 2xx status code when downloading a file".into()));
    }

    let content_disposition = response.headers().get(CONTENT_DISPOSITION);

    let filename = match content_disposition {
        None => match response.url().path().split("/").last() {
            Some(x) => String::from(x),
            None => return Err(ApplicationError::InternalError("Failed to get filename for file".into()))
        },
        Some(header_value) => match header_value.to_str() {
            Ok(header_value) => match content_disposition::parse_content_disposition(header_value).filename_full() {
                Some(filename) => filename,
                None => return Err(ApplicationError::InternalError("Failed to get filename for file".into()))
            }
            Err(_) => return Err(ApplicationError::InternalError("Failed to get filename for file".into()))
        }
    };

    println!("Downloading to {}", filename);

    let full_path = ensure_unique_file_full_filename(dir, &filename);

    let mut file = File::create(&full_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    Ok(full_path)

}

async fn download_files_to_temp_dir(
    sha256s: Vec<String>,
    base_url: &str,
    user_id: i64,
    user_hash: &str,
) -> Result<(PathBuf, Vec<PathBuf>), ApplicationError> {
    let temp_dir = get_temp_dir("download");
    let mut paths = Vec::new();

    for sha256 in sha256s {
        let url = format!("{}/api/v1/blobs/{}/download?user_id={}&user_hash={}", base_url, sha256, user_id, user_hash);
        paths.push(download_file(&url, &temp_dir).await?);
    }

    Ok((temp_dir, paths))
}

#[tauri::command]
pub async fn download_files_and_open_in_folder(
    sha256s: Vec<String>,
    base_url: &str,
    user_id: i64,
    user_hash: &str,
) -> Result<(), ApplicationError> {
    let (temp_dir, _) = download_files_to_temp_dir(sha256s, base_url, user_id, user_hash).await?;

    service::open_folder_in_explorer(&temp_dir);

    Ok(())
}

#[tauri::command]
pub async fn download_files_and_open_in_slicer(
    sha256s: Vec<String>,
    base_url: &str,
    user_id: i64,
    user_hash: &str,
    state: State<'_, TauriAppState>,
) -> Result<(), ApplicationError> {
    if let Some(slicer) = &state.get_configuration().slicer {
        let temp_dir = download_files_to_temp_dir(sha256s, base_url, user_id, user_hash).await?;
        slicer.open(temp_dir.1, &state.app_state).await?;
    }

    Ok(())
}

async fn login(
    token: &str,
    base_url: &str,
) -> Result<Arc<Jar>, ApplicationError> {
    let jar = Arc::new(Jar::default());
    let client = reqwest::ClientBuilder::new()
        .cookie_provider(Arc::clone(&jar))
        .build()
        .unwrap();

    let url = format!("{}/api/v1/login/token", base_url);

    let response = client.post(&url)
        .json(&serde_json::json!({
            "token": token
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(ApplicationError::InternalError("Failed to login with token".into()));
    }

    Ok(jar)
}

async fn logout(
    jar: Arc<Jar>,
    base_url: &str,
) -> Result<(), ApplicationError> {
    let client = reqwest::ClientBuilder::new()
        .cookie_provider(Arc::clone(&jar))
        .build()
        .unwrap();

    let url = format!("{}/api/v1/logout", base_url);

    let response = client.post(&url)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(ApplicationError::InternalError("Failed to logout".into()));
    }

    Ok(())
}

const MAX_CONCURRENT_UPLOADS: usize = 4;

async fn get_ids(
    response: reqwest::Response,
) -> Result<Vec<i64>, ApplicationError> {
    if response.status().is_success() {
        let model_ids: Vec<i64> = response.json().await?;

        if model_ids.len() == 0 {
            println!("Upload returned no model IDs");
            return Err(ApplicationError::InternalError("No model IDs returned after upload".into()));
        }

        Ok(model_ids)
    } else {
        println!("Upload failed with status: {} and response '{}'", response.status(), response.text().await.unwrap_or_default());
        Err(ApplicationError::InternalError("Failed to upload model".into()))
    }
}

async fn process_uploads(
    jar: Arc<Jar>,
    base_url: &str,
    paths: &mut Vec<DirectoryScanModel>,
    source_url: Option<String>,
    app_state: &TauriAppState,
    app_handle: &AppHandle,
) -> Result<ImportState, ApplicationError> {
    let mut import_state = tauri_import_state::import_state_new_tauri(source_url, false, false, false, app_state, app_handle);
    import_state.update_status(ImportStatus::ProcessingModels);
    import_state.update_total_model_count(paths.len());

    let client = reqwest::ClientBuilder::new()
        .cookie_provider(Arc::clone(&jar))
        .build()
        .unwrap();

    let url = format!("{}/api/v1/models", base_url);
    let mut futures = JoinSet::new();

    let mut results = Vec::new();

    for path in &mut *paths {
        let mut form = reqwest::multipart::Form::new();

        if let Some(source_url) = &import_state.origin_url {
            form = form.text("source_url", source_url.clone());
        }

        if (!path.path.exists()) {
            println!("Warning: Path {} does not exist, skipping upload", path.path.display());
        }

        form = form.file("file", &path.path).await?;

        {
            let path = PathBuf::from(&path.path);
            let client = client.clone();
            let url = url.clone();
            futures.spawn(async move {
                (
                    path,
                    client.post(&url)
                    .multipart(form)
                    .send()
                    .await
                )
            });
        }

        if futures.len() >= MAX_CONCURRENT_UPLOADS {
            if let Some(res) = futures.join_next().await {
                match res {
                    Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                    Err(err) => return Err(ApplicationError::InternalError(format!("Upload task failed: {}", err))),
                    Ok(response) => {
                        let path = response.0;
                        let response = response.1?;
                        let ids = get_ids(response).await?;
                    
                        for model_id in &ids {
                            import_state.add_model_id_to_current_set(*model_id);
                        }

                        import_state.update_total_model_count(import_state.model_count + ids.len() - 1);

                        results.push((path, ids));
                    }
                }
            }
        }
    }

    for response in futures.join_all().await {
        let path = response.0;
        let response = response.1?;
        let ids = get_ids(response).await?;
        results.push((path, ids));
    }

    for result in results {
        let path = result.0;
        let model_ids = result.1;

        // Not super efficient, fix later
        let path = paths.iter_mut().find(|p| p.path == path).unwrap();

        path.model_ids = Some(model_ids);
    }

    import_state.update_status(ImportStatus::Finished);
    Ok(import_state)
}

#[derive(Serialize)]
pub struct UploadResult {
    pub import_state: ImportState,
    pub uploaded_models: Vec<DirectoryScanModel>,
}

#[tauri::command]
pub async fn upload_models_to_remote_server(
    paths: Vec<String>,
    source_url: Option<String>,
    recursive: bool,
    open_in_slicer: bool,
    app_state: State<'_, TauriAppState>,
    app_handle: AppHandle,
) -> Result<UploadResult, ApplicationError> {
    let user = app_state.get_current_user();
    let base_url = match user.sync_url {
        Some(url) => url,
        None => return Err(ApplicationError::InternalError("No sync URL set for user".into())),
    };
    let token = match user.sync_token {
        Some(token) => token,
        None => return Err(ApplicationError::InternalError("No sync token set for user".into())),
    };
    let paths: Vec<PathBuf> = paths.iter().map(|p| PathBuf::from(p)).collect();

    let jar = login(&token, &base_url).await?;
    let mut scan = import_service::expand_paths(&paths, recursive).await?;
    let import_state = process_uploads(Arc::clone(&jar), &base_url, &mut scan, source_url, &app_state, &app_handle).await?;

    logout(jar, &base_url).await?;

    if open_in_slicer && scan.len() > 0 {
        if let Some(slicer) = &app_state.get_configuration().slicer {
            let model_paths: Vec<PathBuf> = scan.iter().map(|m| m.path.clone()).collect();
            slicer.open(model_paths, &app_state.app_state).await?;
        }
    }

    Ok(UploadResult {
        import_state,
        uploaded_models: scan,
    })
}

#[tauri::command]
pub async fn expand_paths(
    paths: Vec<String>,
    recursive: bool,
) -> Result<Vec<DirectoryScanModel>, ApplicationError> {
    let paths: Vec<PathBuf> = paths.iter().map(|p| PathBuf::from(p)).collect();

    Ok(import_service::expand_paths(&paths, recursive).await?)
}

#[tauri::command]
pub async fn get_file_bytes(
    path: String,
) -> Result<Response, ApplicationError> {
    let path = PathBuf::from(path);

    if !(is_any_supported_extension(&path) || path.extension().map(|e| e.to_string_lossy().to_lowercase().ends_with("zip")).unwrap_or(false)) {
        return Err(ApplicationError::InternalError("Unsupported file extension for getting bytes".into()));
    }

    let bytes = tokio::fs::read(&path).await?;

    Ok(Response::new(bytes))
}

