use crate::util::{cleanse_evil_from_name, convert_zip_to_extension, is_zipped_file_extension};
use crate::{db::model::Model, error::ApplicationError};
use chrono::Utc;
use std::{fs::File, path::PathBuf};

use super::app_state::AppState;

pub fn export_to_temp_folder(
    models: Vec<Model>,
    app_state: &AppState,
    lazy: bool,
    action: &str,
) -> Result<(PathBuf, Vec<PathBuf>), ApplicationError> {
    let temp_dir = std::env::temp_dir().join(format!(
        "meshorganiser_{}_action_{}",
        action,
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&temp_dir)?;

    let paths: Vec<PathBuf> = models
        .iter()
        .map(|f| get_path_from_model(&temp_dir, f, &app_state, lazy).unwrap())
        .collect();

    if app_state.get_configuration().export_metadata {
        let metadata_path = temp_dir.join("metadata.json");
        let metadata_file = File::create(&metadata_path)?;
        serde_json::to_writer_pretty(metadata_file, &models)?;
    }

    Ok((temp_dir, paths))
}

pub fn get_bytes_from_model(
    model: &Model,
    app_state: &AppState
) -> Result<Vec<u8>, ApplicationError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let src_file_path = base_dir.join(format!("{}.{}", model.sha256, model.filetype));
    let mut file = File::open(src_file_path)?;
    let mut buffer = Vec::new();

    if is_zipped_file_extension(&model.filetype)
    {
        let mut archive = zip::ZipArchive::new(file)?;
        let mut file = archive.by_index(0)?;
        std::io::copy(&mut file, &mut buffer)?;
    } else {
        std::io::copy(&mut file, &mut buffer)?;
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

fn get_path_from_model(
    temp_dir: &PathBuf,
    model: &Model,
    app_state: &AppState,
    lazy: bool,
) -> Result<PathBuf, ApplicationError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let src_file_path = base_dir.join(format!("{}.{}", model.sha256, model.filetype));
    let cleansed_name = cleanse_evil_from_name(&model.name);
    let extension = convert_zip_to_extension(&model.filetype);
    let dst_file_path = ensure_unique_file(temp_dir, &cleansed_name, &extension);

    if is_zipped_file_extension(&model.filetype) {
        let file = File::open(src_file_path)?;

        let mut archive = zip::ZipArchive::new(file)?;
        let mut file = archive.by_index(0)?;
        let mut dst_file = File::create(&dst_file_path)?;

        std::io::copy(&mut file, &mut dst_file)?;
        Ok(dst_file_path)
    } else if !lazy {
        std::fs::copy(&src_file_path, &dst_file_path)?;
        Ok(dst_file_path)
    } else {
        Ok(src_file_path)
    }
}