use super::app_state::AppState;
use crate::db::model_group;
use crate::util::{self, read_file_as_text};
use crate::{db::model, error::ApplicationError};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use zip;
use zip::write::SimpleFileOptions;
use crate::util::{convert_extension_to_zip, is_zippable_file_extension};

#[derive(Serialize)]
pub struct CreationResult {
    pub group_id: Option<i64>,
    pub model_ids: Vec<i64>,
}

pub fn import_path(path: &str, app_state: &AppState, app_handle: &AppHandle) -> Result<CreationResult, ApplicationError> {
    let path_buff = PathBuf::from(path);
    let name = util::prettify_file_name(&path_buff);
    let is_step_supported = app_state.get_configuration().allow_importing_step;

    if path_buff.is_dir() {
        return import_models_from_dir(path, &name, &app_state, &app_handle);
    } else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip" {
        return import_models_from_zip(path, &name, &app_state, &app_handle);
    } else if is_supported_extension(&path_buff, is_step_supported) {
        let extension = path_buff.extension().unwrap().to_str().unwrap();
        let size = path_buff.metadata()?.len() as usize;
        let mut file = File::open(&path_buff)?;

        let result = import_single_model(&mut file, extension, size, &name, None, &app_state,)?;

        return Ok(CreationResult {
            group_id: None,
            model_ids: vec![result],
        });
    }

    return Err(ApplicationError::InternalError(String::from(
        "Unsupported file type",
    )));
}

fn import_models_from_dir(
    path: &str,
    group_name: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
) -> Result<CreationResult, ApplicationError> {
    let group_id = model_group::add_empty_group_sync(group_name, &app_state.db);
    let mut model_ids = Vec::new();
    let is_step_supported = app_state.get_configuration().allow_importing_step;
    let mut temp_str;
    let mut link = None;

    for entry in read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file())
    {
        if entry.file_name().take().unwrap() == ".link"
        {
            temp_str = read_file_as_text(&entry)?;
            link = Some(temp_str.as_str());
        }

        if !is_supported_extension(&entry, is_step_supported)
        {
            continue;
        }

        let file_name = util::prettify_file_name(&entry);
        let extension = entry.extension().unwrap().to_str().unwrap();
        let file_size = entry.metadata()?.len() as usize;
        let mut file = File::open(&entry)?;

        let id = import_single_model(&mut file, extension, file_size, &file_name, link, &app_state)?;
        model_ids.push(id);
        let _ = app_handle.emit("import-count", model_ids.len());
    }

    model_group::set_group_id_on_models_sync(Some(group_id), model_ids.clone(), &app_state.db);

    Ok(CreationResult {
        group_id: Some(group_id),
        model_ids: model_ids,
    })
}

fn import_models_from_zip(
    path: &str,
    group_name: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
) -> Result<CreationResult, ApplicationError> {
    let zip_file = File::open(&path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;
    let group_id = model_group::add_empty_group_sync(group_name, &app_state.db);
    let mut model_ids = Vec::new();
    let is_step_supported = app_state.get_configuration().allow_importing_step;
    let mut temp_str;
    let mut link = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if outpath.file_name().take().unwrap() == ".link"
        {
            let mut file_contents: Vec<u8> = Vec::new();
            file.read_to_end(&mut file_contents)?;
            temp_str = String::from_utf8(file_contents).unwrap();
            link = Some(temp_str.as_str());
        }

        if !is_supported_extension(&outpath, is_step_supported) {
            continue;
        }

        if file.is_file() {
            let file_name = util::prettify_file_name(&outpath);
            let extension = outpath.extension().unwrap().to_str().unwrap();
            let file_size = file.size() as usize;

            let id = import_single_model(&mut file, extension, file_size, &file_name, link, &app_state)?;
            model_ids.push(id);
            let _ = app_handle.emit("import-count", model_ids.len());
        }
    }

    model_group::set_group_id_on_models_sync(Some(group_id), model_ids.clone(), &app_state.db);

    Ok(CreationResult {
        group_id: Some(group_id),
        model_ids: model_ids,
    })
}

fn import_single_model<W>(
    reader: &mut W,
    file_type: &str,
    file_size: usize,
    name: &str,
    link : Option<&str>,
    app_state: &AppState,
) -> Result<i64, ApplicationError>
where
    W: Read,
{
    let mut file_contents: Vec<u8> = match file_size {
        0 => Vec::new(),
        val => Vec::with_capacity(val),
    };

    let _ = reader.read_to_end(&mut file_contents)?;

    let mut hasher = Sha256::new();
    hasher.update(&file_contents);
    let bytes = hasher.finalize();
    let hash = String::from(&format!("{:x}", bytes)[0..32]);

    match model::get_model_id_via_sha256_sync(&hash, &app_state.db) {
        Some(id) => return Ok(id),
        None => (),
    }

    let new_extension = convert_extension_to_zip(file_type);

    let final_file_name = PathBuf::from(app_state.get_model_dir()).join(format!("{}.{}", hash, new_extension));

    let mut file_handle = File::create(&final_file_name)?;

    if is_zippable_file_extension(file_type) {
        let mut zip = zip::ZipWriter::new(file_handle);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(format!("{}.{}", name, file_type), options)?;
        zip.write_all(&file_contents)?;
        zip.finish()?;
    } else {
        file_handle.write_all(&file_contents)?;
    }

    let id = model::add_model_sync(
        name,
        &hash,
        new_extension,
        file_size as i64,
        link,
        &app_state.db,
    );

    return Ok(id);
}

fn is_supported_extension(path: &PathBuf, is_step_supported : bool) -> bool {
    match path.extension() {
        Some(ext) => ext == "stl" || ext == "obj" || ext == "3mf" || (is_step_supported && ext == "step"),
        None => false,
    }
}
