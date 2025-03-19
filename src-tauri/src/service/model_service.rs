use crate::db::model_group;
use crate::{db::model, error::ApplicationError};
use super::app_state::AppState;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use tokio::task::spawn_blocking;
use zip;
use std::io::{Read, Write};
use zip::write::SimpleFileOptions;
use crate::util;
use std::sync::Arc;
use std::fs::{File, read_dir};

pub struct CreationResult
{
    pub group_id : Option<i64>,
    pub model_ids : Vec<i64>
}

pub async fn import_path(path : &str, app_state : &AppState) -> Result<CreationResult, ApplicationError> 
{
    let path_buff = PathBuf::from(path);
    let name = util::prettify_file_name(&path_buff);

    if path_buff.is_dir()
    {
        return import_models_from_dir(path, &name, &app_state).await;
    }
    else if path_buff.extension().is_some() && path_buff.extension().unwrap() == "zip"
    {
        return import_models_from_zip(path, &name, &app_state).await;
    }
    else if path_buff.extension().is_some()
    {
        let extension = path_buff.extension().unwrap().to_str().unwrap();
        let size = path_buff.metadata()?.len() as usize;
        // TOOD: This is not async
        let mut file = File::open(&path_buff)?;

        let result = import_single_model(&mut file, extension, size, &name, &app_state).await?;

        return Ok(CreationResult {
            group_id : None,
            model_ids : vec![result]
        });
    }

    return Err(ApplicationError::InternalError);
}

async fn import_models_from_dir(path : &str, group_name : &str, app_state : &AppState) -> Result<CreationResult, ApplicationError>
{
    let group_id = model_group::add_empty_group(group_name, &app_state.db).await;
    let mut model_ids = Vec::new();

    for entry in read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file())
        .filter(|f| match f.extension() {
            Some(ext) => ext == "stl" || ext == "obj" || ext == "3mf",
            None => false
        })
        {
            let file_name = util::prettify_file_name(&entry);
            let extension = entry.extension().unwrap().to_str().unwrap();
            let file_size = entry.metadata()?.len() as usize;
            let mut file = File::open(&entry)?;
            
            let id = import_single_model(&mut file, extension, file_size, &file_name, &app_state).await?;
            model_ids.push(id);
        }


    model_group::set_group_id_on_models(Some(group_id), model_ids.clone(), &app_state.db).await;

    Ok(CreationResult {
        group_id : Some(group_id),
        model_ids : model_ids
    })
}

async fn import_models_from_zip(path: &str, group_name: &str, app_state: &AppState) -> Result<CreationResult, ApplicationError> {
    let zip_file = File::open(&path)?;
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let group_id = model_group::add_empty_group(group_name, &app_state.db).await;
    let mut model_ids = Vec::new();

    // TODO: This is not async
    for i in 0..archive.len()
    {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if file.is_file()
        {
            let file_name = util::prettify_file_name(&outpath);
            let extension = outpath.extension().unwrap().to_str().unwrap();
            let file_size = file.size() as usize;

            let id = import_single_model(&mut file, extension, file_size, &file_name, &app_state).await?;
            model_ids.push(id);
        }
    }

    model_group::set_group_id_on_models(Some(group_id), model_ids.clone(), &app_state.db).await;

    Ok(CreationResult {
        group_id : Some(group_id),
        model_ids : model_ids
    })
}

// TODO: Implement
pub async fn import_deep_link(url : &str, app_state : &AppState) -> Result<CreationResult, ApplicationError>
{
    Ok(CreationResult {
        group_id : None,
        model_ids : Vec::new()
     })
}

async fn import_single_model<W>(reader : &mut W, file_type : &str, file_size : usize, name : &str, app_state : &AppState) -> Result<i64, ApplicationError>
    where W: Read
{
    
    let mut file_contents_src: Vec<u8> = match file_size
    {
        0 => Vec::new(),
        val => Vec::with_capacity(val)
    };

    // TODO: Make this async
    let bytes_read = reader.read_to_end(&mut file_contents_src)?;

    let file_contents = Arc::new(file_contents_src);
    let file_contents_clone = Arc::clone(&file_contents);

    let hash = spawn_blocking(move || -> String {
        let mut hasher = Sha256::new();
        hasher.update(file_contents_clone.as_ref());
        let bytes = hasher.finalize();
        String::from(&format!("{:x}", bytes)[0..32])
    }).await?;

    match model::get_model_id_via_sha256(&hash, &app_state.db).await
    {
        Some(id) => return Ok(id),
        None => ()
    }

    let final_file_name = match file_type
    {
        "stl" => PathBuf::from(app_state.get_model_dir()).join(format!("{}.stl.zip", hash)),
        _ => PathBuf::from(app_state.get_model_dir()).join(format!("{}.{}", hash, file_type))
    };

    let file_type_clone = String::from(file_type);
    let name_clone = String::from(name);    

    spawn_blocking(move || -> Result<(), ApplicationError> {
        let mut file_handle = File::create(&final_file_name)?;

        if file_type_clone == "stl"
        {
            let mut zip = zip::ZipWriter::new(file_handle);
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file(format!("{}.stl", name_clone), options)?;
            zip.write_all(&file_contents)?;
            zip.finish()?;
        }
        else 
        {
            file_handle.write_all(&file_contents)?;
        }

        Ok(())
    }).await??;

    let id = model::add_model(name, &hash, &file_type, &app_state.db).await;

    return Ok(id);
}