use super::app_state::AppState;
use crate::configuration::Configuration;
use crate::service::import_state::{ImportState, ImportStatus, ImportedModelsSet};
use crate::util::{self, read_file_as_text};
use crate::util::{convert_extension_to_zip, is_zippable_file_extension};
use crate::error::ApplicationError;
use db::model::{Model, User};
use db::model_db::ModelFilterOptions;
use indexmap::IndexMap;
use itertools::Itertools;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::fs::{self, File, read_dir};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use zip;
use zip::write::SimpleFileOptions;

pub fn import_path(
    path: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
    import_state: &mut ImportState,
) -> Result<(), ApplicationError> {
    import_state.status = ImportStatus::ProcessingModels;
    import_state.emit_all(app_handle);

    let configuration = app_state.get_configuration();
    let model_count = get_model_count(path, &configuration, import_state.recursive)?;
    import_state.update_total_model_count(model_count, app_handle);

    match import_path_inner(path, app_state, app_handle, import_state) {
        Ok(()) => {
            import_state.update_status(ImportStatus::FinishedModels, app_handle);
            Ok(())
        }
        Err(application_error) => {
            import_state.set_failure(application_error.to_string(), app_handle);
            Err(application_error)
        }
    }
}

pub fn get_model_count(
    path: &str,
    configuration: &Configuration,
    recursive: bool,
) -> Result<usize, ApplicationError> {
    let path_buff = PathBuf::from(path);

    if path_buff.is_dir() {
        if recursive {
            get_model_count_from_dir_recursive(path, configuration)
        } else {
            get_model_count_from_dir(path, configuration)
        }
    } else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip" {
        get_model_count_from_zip(path, configuration)
    } else if is_supported_extension(&path_buff, &configuration) {
        Ok(1)
    } else {
        Err(ApplicationError::InternalError(String::from(
            "Unsupported file type",
        )))
    }
}

pub fn import_path_inner(
    path: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
    import_state: &mut ImportState,
) -> Result<(), ApplicationError> {
    let path_buff = PathBuf::from(path);
    let name = util::prettify_file_name(&path_buff, path_buff.is_dir());
    let configuration = app_state.get_configuration();

    if path_buff.is_dir() {
        if import_state.recursive {
            import_models_from_dir_recursive(&path_buff, app_state, app_handle, import_state)?;
        } else {
            import_models_from_dir(path, app_state, app_handle, import_state, name.clone())?;
        }
    } else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip" {
        import_models_from_zip(path, app_state, app_handle, import_state, name.clone())?;
    } else if is_supported_extension(&path_buff, &configuration) {
        let extension = path_buff.extension().unwrap().to_str().unwrap();
        let size = path_buff.metadata()?.len() as usize;

        {
            let mut file = File::open(&path_buff)?;
            let id = import_single_model(
                &mut file,
                extension,
                size,
                &name,
                import_state.origin_url.clone(),
                app_state,
            )?;
            import_state.add_model_id_to_current_set(id, app_handle);
        }

        if import_state.delete_after_import {
            let _ = fs::remove_file(&path_buff);
        }
    } else {
        return Err(ApplicationError::InternalError(String::from(
            "Unsupported file type",
        )));
    }

    add_labels_by_keywords(&import_state.imported_models, app_state);

    Ok(())
}

pub fn add_labels_by_keywords(new_models: &Vec<ImportedModelsSet>, app_state: &AppState) {
    let db = &app_state.db;
    let model_ids = new_models
        .iter()
        .flat_map(|r| r.model_ids.iter())
        .cloned()
        .unique()
        .collect::<Vec<i64>>();
    
    let models = tauri::async_runtime::block_on(async {
        db::model_db::get_models_via_ids(db, &User::default(), model_ids).await
    });

    let models = match models {
        Ok(m) => m,
        Err(_) => return,
    };

    let all_keywords = tauri::async_runtime::block_on(async {
        db::label_keyword_db::get_all_keywords(db, &db::model::User::default())
            .await
    });

    let all_keywords = match all_keywords {
        Ok(k) => k,
        Err(_) => return,
    };
    
    let mut all_keywords_map: IndexMap<String, Vec<i64>> = IndexMap::new();

    for (label_id, keywords) in all_keywords.iter() {
        for keyword in keywords {
            if all_keywords_map.contains_key(&keyword.name) {
                all_keywords_map[&keyword.name].push(*label_id);
            } else {
                all_keywords_map.insert(keyword.name.clone(), vec![*label_id]);
            }
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
            tauri::async_runtime::block_on(async {
                let _ = db::label_db::add_labels_on_models(db, &db::model::User::default(), &label_ids, &[model.id], true).await;
            });
        }
    }
}

fn import_models_from_dir_recursive(
    path: &PathBuf,
    app_state: &AppState,
    app_handle: &AppHandle,
    import_state: &mut ImportState,
) -> Result<(), ApplicationError> {
    let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(path)?.map(|x| x.unwrap()).collect();
    let configuration = app_state.get_configuration();

    for folder in entries.iter().filter(|f| f.path().is_dir()) {
        let _ =
            import_models_from_dir_recursive(&folder.path(), app_state, app_handle, import_state);
    }

    if entries
        .iter()
        .filter(|f| f.path().is_file())
        .any(|f| is_supported_extension(&f.path(), &configuration))
    {
        let group_name = util::prettify_file_name(path, true);

        let _ = import_models_from_dir(
            path.to_str().unwrap(),
            app_state,
            app_handle,
            import_state,
            group_name,
        );
    }

    Ok(())
}

fn import_models_from_dir(
    path: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
    import_state: &mut ImportState,
    group_name: String,
) -> Result<(), ApplicationError> {
    let configuration = app_state.get_configuration();

    import_state.add_new_import_set(Some(group_name), app_handle);

    let origin_url = import_state.origin_url.clone();
    let delete_after_import = import_state.delete_after_import;
    let import_state_mutex = Mutex::new(import_state);

    let entries: Vec<PathBuf> = read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file())
        .collect();

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(configuration.core_parallelism)
        .build()
        .unwrap();

    thread_pool.install(|| {
        entries.par_iter().for_each(|entry| {
            let link;

            if entry.file_name().take().unwrap() == ".link" {
                link = Some(read_file_as_text(&entry).unwrap());
            } else {
                link = origin_url.clone();
            }

            if !is_supported_extension(&entry, &configuration) {
                return;
            }

            let file_name = util::prettify_file_name(&entry, false);
            let extension = entry.extension().unwrap().to_str().unwrap();
            let file_size = entry.metadata().unwrap().len() as usize;

            {
                let mut file = File::open(&entry).unwrap();

                let id = import_single_model(
                    &mut file, extension, file_size, &file_name, link, app_state,
                )
                .unwrap();

                let import_state = &mut import_state_mutex.lock().unwrap();
                import_state.add_model_id_to_current_set(id, app_handle);
            }

            if delete_after_import {
                let _ = fs::remove_file(&entry);
            }
        });
    });

    import_state_mutex
        .lock()
        .unwrap()
        .create_group_from_current_set(app_state)?;

    Ok(())
}

fn import_models_from_zip(
    path: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
    import_state: &mut ImportState,
    group_name: String,
) -> Result<(), ApplicationError> {
    let configuration = app_state.get_configuration();
    let mut temp_str;
    let mut link;
    import_state.add_new_import_set(Some(group_name), app_handle);

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
                link = Some(temp_str);
            } else {
                link = import_state.origin_url.clone();
            }

            if !is_supported_extension(&outpath, &configuration) {
                continue;
            }

            if file.is_file() {
                let file_name = util::prettify_file_name(&outpath, false);
                let extension = outpath.extension().unwrap().to_str().unwrap();
                let file_size = file.size() as usize;

                let id = import_single_model(
                    &mut file, extension, file_size, &file_name, link, app_state,
                )?;
                import_state.add_model_id_to_current_set(id, app_handle);
            }
        }
    }

    if import_state.delete_after_import {
        let _ = fs::remove_file(path);
    }

    import_state.create_group_from_current_set(app_state)?;

    Ok(())
}

fn import_single_model<W>(
    reader: &mut W,
    file_type: &str,
    file_size: usize,
    name: &str,
    link: Option<String>,
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

    let existing_id = tauri::async_runtime::block_on(async {
        db::model_db::get_model_id_via_sha256(&app_state.db, &hash)
            .await
    })?;
    
    if let Some(id) = existing_id {
        return Ok(id);
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

    let blob_id = tauri::async_runtime::block_on(async {
        db::blob_db::add_or_create_blob_using_sha256(&app_state.db, &hash, &new_extension, file_size as i64)
            .await
    })?;

    let id = tauri::async_runtime::block_on(async {
        db::model_db::add_model(
            &app_state.db,
            &db::model::User::default(),
            name,
            blob_id,
            link.as_deref(),
            true,
        )
        .await
    })?;

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

fn get_model_count_from_dir_recursive(
    path: &str,
    configuration: &Configuration,
) -> Result<usize, ApplicationError> {
    let entries: Vec<std::fs::DirEntry> = read_dir(path)?.map(|x| x.unwrap()).collect();
    let mut count = 0;

    for folder in entries.iter().filter(|f| f.path().is_dir()) {
        count +=
            get_model_count_from_dir_recursive(folder.path().to_str().unwrap(), configuration)?;
    }

    count += get_model_count_from_dir(path, configuration)?;

    Ok(count)
}

fn get_model_count_from_dir(
    path: &str,
    configuration: &Configuration,
) -> Result<usize, ApplicationError> {
    let size = read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file() && is_supported_extension(&f, &configuration))
        .count();

    Ok(size)
}

fn get_model_count_from_zip(
    path: &str,
    configuration: &Configuration,
) -> Result<usize, ApplicationError> {
    let zip_file = File::open(&path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;
    let mut count = 0;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if is_supported_extension(&outpath, &configuration) {
            count += 1;
        }
    }

    Ok(count)
}
