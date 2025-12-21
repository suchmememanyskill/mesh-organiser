use crate::ASYNC_MULT;
use crate::util::{cleanse_evil_from_name, convert_zip_to_extension, is_zipped_file_extension};
use crate::service_error::ServiceError;
use async_zip::{Compression, ZipEntryBuilder};
use async_zip::tokio::read::seek::ZipFileReader;
use async_zip::tokio::write::ZipFileWriter;
use db::blob_db;
use db::model::{Blob, Model};
use chrono::Utc;
use futures::AsyncWriteExt;
use itertools::Itertools;
use tokio::fs::File;
use tokio::io::{BufReader, BufWriter};
use tokio::task::JoinSet;
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt, TokioAsyncReadCompatExt};
use std::panic;
use std::path::PathBuf;
use std::collections::HashSet;

use super::app_state::AppState;

pub fn get_temp_dir(
    action: &str,
) -> PathBuf {
    let temp_dir = std::env::temp_dir().join(format!(
        "meshorganiser_{}_action_{}",
        action,
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&temp_dir).unwrap();
    temp_dir
}

pub fn get_model_path_for_blob(
    blob: &Blob,
    app_state: &AppState,
) -> PathBuf {
    if let Some(disk_path) = &blob.disk_path {
        PathBuf::from(disk_path)
    } else {
        let base_dir = app_state.get_model_dir();
        base_dir.join(format!("{}.{}", blob.sha256, blob.filetype))
    }
}

pub fn get_image_path_for_blob(
    blob: &Blob,
    app_state: &AppState,
) -> PathBuf {
    let base_dir = app_state.get_image_dir();
    base_dir.join(format!("{}.png", blob.sha256))
}

pub async fn export_to_temp_folder(
    mut models: Vec<Model>,
    app_state: &AppState,
    lazy: bool,
    action: &str,
) -> Result<(PathBuf, Vec<PathBuf>), ServiceError> {
    let configuration = app_state.get_configuration();
    let temp_dir = get_temp_dir(action);

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

fn name_collection_of_models(models : &[Model]) -> String {
    let set : Vec<i64> = models.iter().map(|m| m.group.as_ref().map(|g| g.id).unwrap_or(-1)).unique().collect();

    if set.len() == 1 && set[0] > 0 {
        return cleanse_evil_from_name(&models[0].group.as_ref().unwrap().name);
    }

    cleanse_evil_from_name(&format!("{}{}", models.iter().take(5).map(|m| &m.name).join("+"), if models.len() > 5 { format!("+{} more...", models.len() - 5) } else { "".to_string() }))
}

pub struct ExportZipResult {
    pub temp_dir: PathBuf,
    pub zip_path: PathBuf,
}

pub async fn export_zip_to_temp_folder(
    models: Vec<Model>,
    app_state: &AppState,
) -> Result<ExportZipResult, ServiceError> {
    let configuration = app_state.get_configuration();
    let temp_dir = get_temp_dir("export_zip");

    let filename = format!("{}.zip", name_collection_of_models(&models));
    let filepath = temp_dir.join(filename);

    let mut file = File::create(&filepath).await?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    // TODO: Better way to handle metadata
    if configuration.export_metadata {
        let mut buffer: Vec<u8> = Vec::new();
        serde_json::to_writer_pretty(&mut buffer, &models)?;
        let builder = ZipEntryBuilder::new("metadata.json".into(), Compression::Deflate);
        writer.write_entry_whole(builder, &buffer).await?;
    }

    for model in models {
        let cleansed_name = cleanse_evil_from_name(&model.name);
        let extension = convert_zip_to_extension(&model.blob.filetype);
        let builder = ZipEntryBuilder::new(format!("{}.{}", cleansed_name, extension).into(), Compression::Deflate);
        let mut stream_writer = writer.write_entry_stream(builder).await?;

        // TODO: Find a way to reuse this
        let src_file_path = get_model_path_for_blob(&model.blob, app_state);
        let model_file = File::open(src_file_path).await?;

        if is_zipped_file_extension(&model.blob.filetype) {
            let mut buffered_reader = BufReader::new(model_file);
            let mut zip = ZipFileReader::with_tokio(&mut buffered_reader).await?;
            let mut file = zip.reader_with_entry(0).await?;
            
            futures::io::copy(&mut file, &mut stream_writer).await?;
        } else {
            let mut model_file = model_file.compat();
            futures::io::copy(&mut model_file, &mut stream_writer).await?;
        }

        stream_writer.close().await?; 
        println!("Added model {} to zip", model.name);
    }
    
    writer.close().await?;

    Ok(ExportZipResult {
        temp_dir,
        zip_path: filepath,
    })
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
    let src_file_path = get_model_path_for_blob(blob, app_state);
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

pub fn ensure_unique_file_full_filename(base_path: &PathBuf, file_name : &str) -> PathBuf {
    let extension = file_name.split(".").last().unwrap();
    let base_file_name = &file_name[..file_name.len() - extension.len() - 1];

    ensure_unique_file(base_path, base_file_name, extension)
}

pub fn ensure_unique_file(base_path: &PathBuf, file_name: &str, extension: &str) -> PathBuf {
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
    let src_file_path = get_model_path_for_blob(&model.blob, app_state);
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
    let blobs = blob_db::get_and_delete_dead_blobs(&app_state.db).await?;

    for blob in blobs {
        let model_path = get_model_path_for_blob(&blob, app_state);
        let image_path = get_image_path_for_blob(&blob, app_state);

        if blob.disk_path.is_none() && model_path.exists() {
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