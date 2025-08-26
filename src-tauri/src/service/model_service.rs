use super::app_state::AppState;
use crate::configuration::Configuration;
use crate::db::{label, label_keywords, model_group};
use crate::util::{self, read_file_as_text};
use crate::util::{convert_extension_to_zip, is_zippable_file_extension};
use crate::{db::model, error::ApplicationError};
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{self, read_dir, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use zip;
use zip::write::SimpleFileOptions;

#[derive(Serialize)]
pub struct CreationResult {
    pub group_id: Option<i64>,
    pub model_ids: Vec<i64>,
}

pub fn import_path(
    path: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
    recursive: bool,
    delete_imported: bool,
) -> Result<Vec<CreationResult>, ApplicationError> {
    let path_buff = PathBuf::from(path);
    let name = util::prettify_file_name(&path_buff, path_buff.is_dir());
    let configuration = app_state.get_configuration();
    let creation_result: Vec<CreationResult>;

    if path_buff.is_dir() {
        if recursive {
            creation_result = import_models_from_dir_recursive(
                &path_buff,
                app_state,
                app_handle,
                delete_imported,
            )?;
        } else {
            let result =
                import_models_from_dir(path, &name, &app_state, &app_handle, delete_imported)?;
            creation_result = vec![result];
        }
    } else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip" {
        let result = import_models_from_zip(path, &name, &app_state, &app_handle, delete_imported)?;
        creation_result = vec![result];
    } else if is_supported_extension(&path_buff, &configuration) {
        let extension = path_buff.extension().unwrap().to_str().unwrap();
        let size = path_buff.metadata()?.len() as usize;
        let result;

        {
            let mut file = File::open(&path_buff)?;
            result = import_single_model(&mut file, extension, size, &name, None, &app_state)?;
        }

        if delete_imported {
            let _ = fs::remove_file(&path_buff);
        }

        creation_result = vec![CreationResult {
            group_id: None,
            model_ids: vec![result],
        }];
    } else {
        return Err(ApplicationError::InternalError(String::from(
            "Unsupported file type",
        )));
    }

    add_labels_by_keywords(&creation_result, app_state);

    Ok(creation_result)
}

pub fn add_labels_by_keywords(new_models: &Vec<CreationResult>, app_state: &AppState) {
    let db = &app_state.db;
    let model_ids = new_models
        .iter()
        .flat_map(|r| r.model_ids.iter())
        .cloned()
        .unique()
        .collect::<Vec<i64>>();
    let models = model::get_models_by_id_sync(model_ids, db);

    let all_keywords = label_keywords::get_all_keywords_sync(db);
    let mut all_keywords_map: IndexMap<String, Vec<i64>> = IndexMap::new();

    for keyword in all_keywords.iter() {
        if all_keywords_map.contains_key(&keyword.name) {
            all_keywords_map[&keyword.name].push(keyword.label_id);
        } else {
            all_keywords_map.insert(keyword.name.clone(), vec![keyword.label_id]);
        }
    }

    for model in models.iter() {
        let mut name_parts: Vec<String> = model
            .name
            .split(|c: char| !c.is_alphanumeric())
            .map(|s| s.to_lowercase())
            .collect();

        if let Some(group) = &model.group {
            name_parts.extend(
                group
                    .name
                    .split(|c: char| !c.is_alphanumeric())
                    .map(|s| s.to_lowercase()),
            );
        }

        let label_ids: Vec<i64> = name_parts
            .iter()
            .flat_map(|part| {
                if all_keywords_map.contains_key(part) {
                    return all_keywords_map[part].clone();
                }

                return vec![];
            })
            .filter(|l| {
                !model
                    .labels
                    .iter()
                    .any(|existing_labels| existing_labels.id == *l)
            })
            .unique()
            .collect();

        if !label_ids.is_empty() {
            label::add_labels_on_model_sync(label_ids, model.id, db);
        }
    }
}

fn import_models_from_dir_recursive(
    path: &PathBuf,
    app_state: &AppState,
    app_handle: &AppHandle,
    delete_imported: bool,
) -> Result<Vec<CreationResult>, ApplicationError> {
    let mut results: Vec<CreationResult> = vec![];
    let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(path)?.map(|x| x.unwrap()).collect();
    let configuration = app_state.get_configuration();

    for folder in entries.iter().filter(|f| f.path().is_dir()) {
        if let Ok(result) =
            import_models_from_dir_recursive(&folder.path(), app_state, app_handle, delete_imported)
        {
            results.extend(result);
        }
    }

    if entries
        .iter()
        .filter(|f| f.path().is_file())
        .any(|f| is_supported_extension(&f.path(), &configuration))
    {
        let group_name = util::prettify_file_name(path, true);

        if let Ok(result) = import_models_from_dir(
            path.to_str().unwrap(),
            &group_name,
            app_state,
            app_handle,
            delete_imported,
        ) {
            results.push(result);
        }
    }

    Ok(results)
}

fn import_models_from_dir(
    path: &str,
    group_name: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
    delete_imported: bool,
) -> Result<CreationResult, ApplicationError> {
    let mut model_ids = Vec::new();
    let configuration = app_state.get_configuration();
    let mut temp_str;
    let mut link = None;
    let _ = app_handle.emit("import-count", 0 as usize);
    let _ = app_handle.emit("import-group", group_name);

    for entry in read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file())
    {
        if entry.file_name().take().unwrap() == ".link" {
            temp_str = read_file_as_text(&entry)?;
            link = Some(temp_str.as_str());
        }

        if !is_supported_extension(&entry, &configuration) {
            continue;
        }

        let file_name = util::prettify_file_name(&entry, false);
        let extension = entry.extension().unwrap().to_str().unwrap();
        let file_size = entry.metadata()?.len() as usize;
        let id;

        {
            let mut file = File::open(&entry)?;

            id = import_single_model(
                &mut file, extension, file_size, &file_name, link, &app_state,
            )?;
        }

        model_ids.push(id);

        if delete_imported {
            let _ = fs::remove_file(&entry);
        }

        let _ = app_handle.emit("import-count", model_ids.len());
    }

    if model_ids.is_empty() {
        return Err(ApplicationError::InternalError(String::from(
            "No models found in directory",
        )));
    }

    let group_id = model_group::add_empty_group_sync(group_name, &app_state.db);
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
    delete_imported: bool,
) -> Result<CreationResult, ApplicationError> {
    let mut model_ids = Vec::new();
    let configuration = app_state.get_configuration();
    let mut temp_str;
    let mut link = None;
    let _ = app_handle.emit("import-count", 0 as usize);
    let _ = app_handle.emit("import-group", group_name);

    {
        let zip_file = File::open(&path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };

            if outpath.file_name().take().unwrap() == ".link" {
                let mut file_contents: Vec<u8> = Vec::new();
                file.read_to_end(&mut file_contents)?;
                temp_str = String::from_utf8(file_contents).unwrap();
                link = Some(temp_str.as_str());
            }

            if !is_supported_extension(&outpath, &configuration) {
                continue;
            }

            if file.is_file() {
                let file_name = util::prettify_file_name(&outpath, false);
                let extension = outpath.extension().unwrap().to_str().unwrap();
                let file_size = file.size() as usize;

                let id = import_single_model(
                    &mut file, extension, file_size, &file_name, link, &app_state,
                )?;
                model_ids.push(id);
                let _ = app_handle.emit("import-count", model_ids.len());
            }
        }
    }

    if delete_imported {
        let _ = fs::remove_file(path);
    }

    if model_ids.is_empty() {
        return Err(ApplicationError::InternalError(String::from(
            "No models found in zip file",
        )));
    }

    let group_id = model_group::add_empty_group_sync(group_name, &app_state.db);
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
    link: Option<&str>,
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

    let final_file_name =
        PathBuf::from(app_state.get_model_dir()).join(format!("{}.{}", hash, &new_extension));

    let mut file_handle = File::create(&final_file_name)?;

    if is_zippable_file_extension(file_type) {
        let mut zip = zip::ZipWriter::new(file_handle);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(format!("{}.{}", name, file_type.to_lowercase()), options)?;
        zip.write_all(&file_contents)?;
        zip.finish()?;
    } else {
        file_handle.write_all(&file_contents)?;
    }

    let id = model::add_model_sync(
        name,
        &hash,
        &new_extension,
        file_size as i64,
        link,
        &app_state.db,
    );

    return Ok(id);
}

fn is_supported_extension(path: &PathBuf, configuration: &Configuration) -> bool {
    match path.extension() {
        Some(ext) => {
            let lowercase = ext.to_str().unwrap().to_lowercase();
            lowercase == "stl"
                || lowercase == "obj"
                || lowercase == "3mf"
                || (configuration.allow_importing_gcode && lowercase == "gcode")
                || (configuration.allow_importing_step && lowercase == "step")
        }
        None => false,
    }
}
