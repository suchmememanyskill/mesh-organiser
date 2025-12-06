use super::app_state::AppState;
use crate::ASYNC_MULT;
use crate::configuration::Configuration;
use crate::import_state::{ImportState, ImportStatus, ImportedModelsSet};
use crate::util::{self, read_file_as_text};
use crate::util::{convert_extension_to_zip, is_zippable_file_extension};
use async_zip::ZipEntryBuilder;
use async_zip::tokio::read;
use async_zip::tokio::read::seek::ZipFileReader;
use async_zip::tokio::write::ZipFileWriter;
use db::{blob_db, label_db, label_keyword_db, model_db};
use db::model::{Model, User};
use db::model_db::ModelFilterOptions;
use indexmap::IndexMap;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::task::{JoinSet, spawn_blocking};
use std::fs::{self, read_dir};
use std::io::{Read, Write};
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio_util::{io::ReaderStream, compat::FuturesAsyncReadCompatExt};
use crate::service_error::ServiceError;

pub async fn import_path(
    path: &str,
    app_state: &AppState,
    mut import_state: ImportState,
) -> Result<ImportState, ServiceError> {
    if import_state.delete_after_import && import_state.import_as_path {
        return Err(ServiceError::InternalError(String::from(
            "Cannot delete after import when importing as path",
        )));
    }

    let _lock = app_state.import_mutex.lock().await;
    import_state.status = ImportStatus::ProcessingModels;
    import_state.emit_all();

    let configuration = app_state.get_configuration();
    let model_count = get_model_count(path, &configuration, import_state.recursive, &import_state).await?;
    import_state.update_total_model_count(model_count);

    let import_state = Arc::new(Mutex::new(import_state));
    let import_state_clone = Arc::clone(&import_state);

    match import_path_inner(path, app_state, import_state).await {
        Ok(()) => {
            let mut import_state = import_state_clone.lock().await;
            import_state.update_status(ImportStatus::FinishedModels);
            import_state.create_groups_from_all_sets(app_state).await?;

            let import_state = {
                let fake = ImportState::new(None, false, false, false, User::default());
                std::mem::replace(&mut *import_state, fake)
            };

            Ok(import_state)
        }
        Err(application_error) => {
            let mut import_state = import_state_clone.lock().await;
            import_state.set_failure(application_error.to_string());
            Err(application_error)
        }
    }
}

pub async fn get_model_count(
    path: &str,
    configuration: &Configuration,
    recursive: bool,
    import_state: &ImportState,
) -> Result<usize, ServiceError> {
    let path_buff = PathBuf::from(path);

    if path_buff.is_dir() {
        if recursive {
            get_model_count_from_dir_recursive(path, configuration)
        } else {
            get_model_count_from_dir(path, configuration)
        }
    } else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip" {
        if import_state.import_as_path {
            return Err(ServiceError::InternalError(String::from(
                "Cannot import a zip as path",
            )));
        }
        get_model_count_from_zip(path, configuration).await
    } else if is_supported_extension(&path_buff, &configuration) {
        Ok(1)
    } else {
        Err(ServiceError::InternalError(String::from(
            "Unsupported file type",
        )))
    }
}

pub async fn import_path_inner(
    path: &str,
    app_state: &AppState,
    import_state: Arc<Mutex<ImportState>>,
) -> Result<(), ServiceError> {
    let path_buff = PathBuf::from(path);
    let name = util::prettify_file_name(&path_buff, path_buff.is_dir());
    let configuration = app_state.get_configuration();
    let later_import_state = Arc::clone(&import_state);

    let recurisve = {
        let import_state = import_state.lock().await;
        import_state.recursive
    };

    if path_buff.is_dir() {
        if recurisve {
            import_models_from_dir_recursive(&path_buff, app_state, import_state).await?;
        } else {
            import_models_from_dir(path, app_state, import_state, name.clone()).await?;
        }
    } else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip" {
        import_models_from_zip(path, app_state, import_state, name.clone()).await?;
    } else if is_supported_extension(&path_buff, &configuration) {
        let extension = path_buff.extension().unwrap().to_str().unwrap();
        let size = path_buff.metadata()?.len() as usize;
        let mut import_state = import_state.lock().await;

        {
            let mut file = File::open(&path_buff).await?;
            let permanent_disk_path = if import_state.import_as_path {
                Some(path_buff.clone())
            } else {
                None
            };

            let id = import_single_model(
                &mut file,
                extension,
                size,
                &name,
                import_state.origin_url.clone(),
                app_state,
                &import_state.user,
                permanent_disk_path,
            ).await?;
            import_state.add_model_id_to_current_set(id);
        }

        if import_state.delete_after_import {
            let _ = fs::remove_file(&path_buff);
        }
    } else {
        return Err(ServiceError::InternalError(String::from(
            "Unsupported file type",
        )));
    }

    let import_state = later_import_state.lock().await;
    add_labels_by_keywords(&import_state.imported_models, app_state, &import_state).await;

    Ok(())
}

pub async fn add_labels_by_keywords(new_models: &Vec<ImportedModelsSet>, app_state: &AppState, import_state : &ImportState) {
    let db = &app_state.db;
    let model_ids = new_models
        .iter()
        .flat_map(|r| r.model_ids.iter())
        .cloned()
        .unique()
        .collect::<Vec<i64>>();
    
    let models = model_db::get_models_via_ids(db, &import_state.user, model_ids).await;

    let models = match models {
        Ok(m) => m,
        Err(_) => return,
    };

    let all_keywords = label_keyword_db::get_all_keywords(db, &import_state.user).await;

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
            let _ = label_db::add_labels_on_models(db, &import_state.user, &label_ids, &[model.id] , None).await;
        }
    }
}

async fn import_models_from_dir_recursive(
    path: &PathBuf,
    app_state: &AppState,
    import_state: Arc<Mutex<ImportState>>,
) -> Result<(), ServiceError> {
    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };

    let entries: Vec<std::fs::DirEntry> = read_dir.map(|x| x.unwrap()).collect();
    let configuration = app_state.get_configuration();

    for folder in entries.iter().filter(|f| f.path().is_dir()) {
        let import_state = Arc::clone(&import_state);
        Box::pin(import_models_from_dir_recursive(&folder.path(), app_state, import_state)).await?;
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
            import_state,
            group_name,
        ).await;
    }

    Ok(())
}

async fn import_models_from_dir_inner(
    configuration: &Configuration,
    app_state: &AppState,
    path: PathBuf,
    import_state_mutex: Arc<Mutex<ImportState>>,
    user: &User,
    link: &Option<String>,
    delete_after_import: bool,
    import_as_path: bool,
) -> Result<(), ServiceError> {
    if !is_supported_extension(&path, configuration) {
        return Err(ServiceError::InternalError("Unsupported filetype".into()));
    }

    let file_name = util::prettify_file_name(&path, false);
    let extension = path.extension().unwrap().to_str().unwrap();
    let file_size = path.metadata().unwrap().len() as usize;

    let mut file = File::open(&path).await?;
    let permanent_disk_path = if import_as_path {
        Some(path.clone())
    } else {
       None
    };

    let id = import_single_model(
        &mut file, extension, file_size, &file_name, link.clone(), app_state, user, permanent_disk_path
    ).await?;

    {
        let import_state = &mut import_state_mutex.lock().await;
        import_state.add_model_id_to_current_set(id);
    }

    if delete_after_import {
        let _ = fs::remove_file(&path);
    }

    Ok(())
}

async fn import_models_from_dir(
    path: &str,
    app_state: &AppState,
    import_state: Arc<Mutex<ImportState>>,
    group_name: String,
) -> Result<(), ServiceError> {
    let configuration = app_state.get_configuration();
    let user;
    let origin_url;
    let delete_after_import;
    let import_as_path;
    {
        let mut import_state = import_state.lock().await;

        import_state.add_new_import_set(Some(group_name));
        user = import_state.user.clone();
        origin_url = import_state.origin_url.clone();
        delete_after_import = import_state.delete_after_import;
        import_as_path = import_state.import_as_path;
    }
    
    let mut entries: Vec<PathBuf> = read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file())
        .collect();

    let mut futures = JoinSet::new();

    let max = configuration.core_parallelism * ASYNC_MULT;
    let mut active = 0;

    while !entries.is_empty() {
        let entry = match entries.pop() {
            Some(x) => x,
            None => continue,
        };
        let configuration = configuration.clone();
        let app_state = app_state.clone();
        let import_state_mutex = Arc::clone(&import_state);
        let user = user.clone();
        let origin_url = origin_url.clone();
        let delete_after_import = delete_after_import;
        let import_as_path = import_as_path;
        active += 1;

        futures.spawn(async move {
            import_models_from_dir_inner(&configuration, &app_state, entry, import_state_mutex, &user, &origin_url, delete_after_import, import_as_path).await
        });

        if active >= max {
            if let Some(res) = futures.join_next().await {
                match res {
                    Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                    Err(err) => panic!("{err}"),
                    _ => active -= 1,
                }
            }
        }
    }

    futures.join_all().await;

    Ok(())
}

async fn import_models_from_zip(
    path: &str,
    app_state: &AppState,
    import_state: Arc<Mutex<ImportState>>,
    group_name: String,
) -> Result<(), ServiceError> {
    let configuration = app_state.get_configuration();
    let mut import_state = import_state.lock().await;
    import_state.add_new_import_set(Some(group_name));

    if import_state.import_as_path {
        return Err(ServiceError::InternalError(String::from(
            "Cannot import a zip as path",
        )));
    }

    {
        let zip_file = File::open(path).await?;
        let buffered_reader = BufReader::new(zip_file);
        let mut archive = ZipFileReader::with_tokio(buffered_reader).await?;

        let len = archive.file().entries().len();

        for index in 0..len {
            let file = archive.reader_with_entry(index).await?;
            let link = import_state.origin_url.clone();

            if file.entry().dir()? {
                return Err(ServiceError::InternalError("Zip entry is a folder".into()))
            }

            let path = PathBuf::from(file.entry().filename().as_str()?);
            /* -- Revist this at some point
            if path.file_name().take().unwrap() == ".link" {
                let mut file_contents: Vec<u8> = Vec::new();
                file_compat.read_to_end(&mut file_contents).await?;
                let temp_str = String::from_utf8(file_contents).unwrap();
                link.replace(temp_str);
            }*/

            if !is_supported_extension(&path, &configuration) {
                continue;
            }

            let file_name = util::prettify_file_name(&path, false);
            let extension = path.extension().unwrap().to_str().unwrap();
            let file_size = file.entry().uncompressed_size() as usize;
            let mut file_compat = file.compat();

            let id = import_single_model(
                    &mut file_compat, extension, file_size, &file_name, link, app_state, &import_state.user, None
            ).await?;

            import_state.add_model_id_to_current_set(id);
        }
    }

    if import_state.delete_after_import {
        let _ = fs::remove_file(path);
    }

    Ok(())
}

async fn import_single_model<W>(
    reader: &mut W,
    file_type: &str,
    file_size: usize,
    name: &str,
    link: Option<String>,
    app_state: &AppState,
    user: &User,
    permanent_disk_path: Option<PathBuf>,
) -> Result<i64, ServiceError>
where
    W: AsyncRead + Unpin,
{
    let mut file_contents: Vec<u8> = match file_size {
        0 => Vec::new(),
        val => Vec::with_capacity(val),
    };

    reader.read_to_end(&mut file_contents).await?;

    let mut hasher = Sha256::new();
    hasher.update(&file_contents);
    let bytes = hasher.finalize();
    let hash = String::from(&format!("{:x}", bytes)[0..32]);

    let existing_id = model_db::get_model_id_via_sha256(&app_state.db, user, &hash)
            .await?;
    
    if let Some(id) = existing_id {
        return Ok(id);
    }

    let blob_id_optional = blob_db::get_blob_via_sha256(&app_state.db, &hash).await?;

    let blob_id;

    if let Some(blob) = blob_id_optional {
        blob_id = blob.id;
    } else if let Some(permanent_disk_path) = permanent_disk_path {
        blob_id = blob_db::add_blob(&app_state.db, &hash, file_type, file_size as i64, Some(permanent_disk_path.to_str().unwrap().to_string())).await?;
    } else {
        let new_extension = convert_extension_to_zip(file_type);

        let final_file_name =
            PathBuf::from(app_state.get_model_dir()).join(format!("{}.{}", hash, &new_extension));

        let mut file_handle = File::create(&final_file_name).await?;

        if is_zippable_file_extension(file_type) {
            let mut writer = ZipFileWriter::with_tokio(&mut file_handle);
            let builder = ZipEntryBuilder::new(format!("{}.{}", name, file_type.to_lowercase()).into(), async_zip::Compression::Deflate);

            writer.write_entry_whole(builder, &file_contents).await?;
            writer.close().await?;
        } else {
            file_handle.write_all(&file_contents).await?;
        }

        blob_id = blob_db::add_blob(&app_state.db, &hash, &new_extension, file_size as i64, None).await?;
    }

    let id = model_db::add_model(
            &app_state.db,
            user,
            name,
            blob_id,
            link.as_deref(),
            None
        )
        .await?;

    return Ok(id);
}

pub fn is_supported_extension(path: &PathBuf, configuration: &Configuration) -> bool {
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
) -> Result<usize, ServiceError> {
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
) -> Result<usize, ServiceError> {
    let size = read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file() && is_supported_extension(&f, &configuration))
        .count();

    Ok(size)
}

async fn get_model_count_from_zip(
    path: &str,
    configuration: &Configuration,
) -> Result<usize, ServiceError> {
    let file = File::open(path).await?;
    let mut buffered_reader = BufReader::new(file);
    let zip = ZipFileReader::with_tokio(&mut buffered_reader).await?;
    let mut count = 0;

    for entry in zip.file().entries() {
        let path = PathBuf::from(entry.filename().as_str()?);

        if is_supported_extension(&path, configuration)
        {
            count += 1;
        }
    }

    Ok(count)
}
