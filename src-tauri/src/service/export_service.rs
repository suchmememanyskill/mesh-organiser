use crate::util::{convert_zip_to_extension, is_zipped_file_extension};
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

    Ok((temp_dir, paths))
}

fn cleanse_name(name: &str) -> String {
    String::from(
        name.replace("\\", " ")
            .replace("/", " ")
            .replace(":", " ")
            .replace("*", " ")
            .replace("?", " ")
            .replace("\"", " ")
            .replace("<", " ")
            .replace(">", " ")
            .replace("|", " ")
            .trim(),
    )
}

fn get_path_from_model(
    temp_dir: &PathBuf,
    model: &Model,
    app_state: &AppState,
    lazy: bool,
) -> Result<PathBuf, ApplicationError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let src_file_path = base_dir.join(format!("{}.{}", model.sha256, model.filetype));

    if is_zipped_file_extension(&model.filetype) {
        let file = File::open(src_file_path)?;
        let extension = convert_zip_to_extension(&model.filetype);

        let target = temp_dir.join(format!(
            "{}_{}.{}",
            cleanse_name(&model.name),
            model.sha256,
            extension
        ));
        let mut archive = zip::ZipArchive::new(file)?;
        let mut file = archive.by_index(0)?;
        let mut target_file = File::create(&target)?;

        std::io::copy(&mut file, &mut target_file)?;
        Ok(target)
    } else if !lazy {
        let dst_file_path = temp_dir.join(format!(
            "{}_{}.{}",
            cleanse_name(&model.name),
            model.sha256,
            model.filetype
        ));
        std::fs::copy(&src_file_path, &dst_file_path)?;
        Ok(dst_file_path)
    } else {
        Ok(src_file_path)
    }
}
