use crate::ASYNC_MULT;
use crate::util::{cleanse_evil_from_name, convert_zip_to_extension, is_zipped_file_extension};
use crate::service_error::ServiceError;
use async_zip::tokio::read::seek::ZipFileReader;
use db::blob_db;
use db::model::{Blob, Model};
use chrono::Utc;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio::task::JoinSet;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use std::panic;
use std::path::PathBuf;
use std::collections::HashSet;

use super::app_state::AppState;

pub async fn export_to_temp_folder(
    mut models: Vec<Model>,
    app_state: &AppState,
    lazy: bool,
    action: &str,
) -> Result<(PathBuf, Vec<PathBuf>), ServiceError> {
    let configuration = app_state.get_configuration();
    let temp_dir = std::env::temp_dir().join(format!(
        "meshorganiser_{}_action_{}",
        action,
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&temp_dir)?;

    let mut futures = JoinSet::new();

    if configuration.export_metadata {
        let metadata_path = temp_dir.join("metadata.json");
        let metadata_file = std::fs::File::create(&metadata_path)?;
        serde_json::to_writer_pretty(metadata_file, &models)?;
    }

    let mut paths = Vec::with_capacity(models.len());
    let max = configuration.core_parallelism * ASYNC_MULT;
    let mut active = 0;

    while !models.is_empty() {
        let model = match models.pop() {
            Some(x) => x,
            None => continue,
        };
        let temp_dir = temp_dir.clone();
        let app_state = app_state.clone();
        active += 1;

        futures.spawn(async move { 
            let model = model;
            get_path_from_model(&temp_dir, &model, &app_state, lazy).await
        });

        if active >= max {
            if let Some(res) = futures.join_next().await {
                match res {
                    Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                    Err(err) => panic!("{err}"),
                    Ok(res) => {
                        if let Ok(res) = res {
                            paths.push(res);
                        }
                        active -= 1;
                    },
                }
            }
        }
    }

    paths.extend(futures.join_all().await.into_iter().filter(|r| r.is_ok()).map(|r| r.unwrap()));

    Ok((temp_dir, paths))
}

pub async fn get_bytes_from_model(
    model: &Model,
    app_state: &AppState,
) -> Result<Vec<u8>, ServiceError> {
    get_bytes_from_blob(&model.blob, app_state).await
}

pub async fn get_bytes_from_blob(
    blob: &Blob,
    app_state: &AppState,
) -> Result<Vec<u8>, ServiceError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let src_file_path = base_dir.join(format!("{}.{}", blob.sha256, blob.filetype));
    let mut file = File::open(src_file_path).await?;
    let mut buffer = Vec::new();

    if is_zipped_file_extension(&blob.filetype) {
        let mut buffered_reader = BufReader::new(file);
        let mut zip = ZipFileReader::with_tokio(&mut buffered_reader).await?;
        let file = zip.reader_with_entry(0).await?;
        let mut file_compat = file.compat();

        tokio::io::copy(&mut file_compat, &mut buffer).await?;
    } else {
        tokio::io::copy(&mut file, &mut buffer).await?;
    }

    Ok(buffer)
}

fn ensure_unique_file(base_path: &PathBuf, file_name: &str, extension: &str) -> PathBuf {
    let mut counter = 1;
    let mut new_file_name = base_path.join(format!("{}.{}", file_name, extension));

    while new_file_name.exists() {
        new_file_name = base_path.join(format!("{}_{}.{}", file_name, counter, extension));
        counter += 1;
    }

    return new_file_name;
}

pub async fn get_path_from_model(
    temp_dir: &PathBuf,
    model: &Model,
    app_state: &AppState,
    lazy: bool,
) -> Result<PathBuf, ServiceError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let src_file_path = base_dir.join(format!("{}.{}", model.blob.sha256, model.blob.filetype));
    let cleansed_name = cleanse_evil_from_name(&model.name);
    let extension = convert_zip_to_extension(&model.blob.filetype);
    let dst_file_path = ensure_unique_file(temp_dir, &cleansed_name, &extension);

    if is_zipped_file_extension(&model.blob.filetype) {
        let zip_file = File::open(src_file_path).await?;
        let mut buffered_reader = BufReader::new(zip_file);
        let mut zip = ZipFileReader::with_tokio(&mut buffered_reader).await?;
        let file = zip.reader_with_entry(0).await?;
        let mut file_compat = file.compat();

        let mut dst_file = File::create(&dst_file_path).await?;

        tokio::io::copy(&mut file_compat, &mut dst_file).await?;
        Ok(dst_file_path)
    } else if !lazy {
        tokio::fs::copy(&src_file_path, &dst_file_path).await?;
        Ok(dst_file_path)
    } else {
        Ok(src_file_path)
    }
}

pub fn get_size_of_blobs(
    blobs: &Vec<String>, // Sha256's
    app_state: &AppState,
) -> Result<u64, ServiceError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let mut total_size: u64 = 0;
    let hashset = blobs.iter().cloned().collect::<HashSet<String>>();

    for path in base_dir.read_dir()? {
        let path = match path {
            Ok(p) => p,
            Err(_) => continue,
        };

        let f = path.file_name();
        let lossy = f.to_string_lossy();
        let filename = match lossy.split('.').next() {
            Some(name) => name,
            None => continue,
        };

        if !hashset.contains(filename) {
            continue;
        }

        let metadata = match path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.is_file() {
            continue;
        }

        total_size += metadata.len();
    }

    Ok(total_size)
}

pub async fn delete_dead_blobs(
    app_state: &AppState,
) -> Result<(), ServiceError> {
    let model_dir = PathBuf::from(app_state.get_model_dir());
    let image_dir = PathBuf::from(app_state.get_image_dir());
   
    let blobs = blob_db::get_and_delete_dead_blobs(&app_state.db).await?;

    for blob in blobs {
        let model_path = model_dir.join(format!("{}.{}", blob.sha256, blob.filetype));
        let image_path = image_dir.join(format!("{}.png", blob.sha256));

        if model_path.exists() {
            if let Err(e) = std::fs::remove_file(model_path) {
                eprintln!("Failed to remove dead blob model file: {}", e);
            }
        }

        if image_path.exists() {
            if let Err(e) = std::fs::remove_file(image_path) {
                eprintln!("Failed to remove dead blob image file: {}", e);
            }
        }
    }
    Ok(())
}