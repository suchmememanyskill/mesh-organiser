use crate::db::model_group;
use crate::{db::model, error::ApplicationError};
use super::app_state::AppState;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use tokio::{self, fs};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use chrono::Utc;
use tokio::task::spawn_blocking;
use zip;
use std::io::{Read, Seek, Write};
use zip::write::SimpleFileOptions;
use std::io;
use std::fs::{DirEntry};
use crate::util;

pub async fn import_models_from_zip(path: &str, group_name: &str, app_state: AppState) -> Result<(i64, Vec<i64>), ApplicationError> {
    let temp_dir = std::env::temp_dir().join(format!("mesh_zip_{}", Utc::now().timestamp_nanos_opt().unwrap()));
    fs::create_dir_all(&temp_dir).await?;

    {
        let path = path.to_string();

        spawn_blocking(move || -> Result<(), ApplicationError> {
            let file = std::fs::File::open(&path)?;
            let mut archive = zip::ZipArchive::new(file).unwrap();
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let outpath = match file.enclosed_name() {
                    Some(path) => path,
                    None => continue,
                };

                if file.is_dir() {
                    std::fs::create_dir_all(&outpath).unwrap();
                } 
                else 
                {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).unwrap();
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath).unwrap();
                    io::copy(&mut file, &mut outfile).unwrap();
                }
            }
            Ok(())
        }).await??;
    }

    let result = import_models_from_dir(temp_dir.to_str().unwrap(), group_name, app_state).await;

    let temp_dir_clone = temp_dir.clone();
    fs::remove_dir_all(temp_dir_clone).await?;
    
    result
}

pub async fn import_models_from_dir(path : &str, group_name : &str, app_state : AppState) -> Result<(i64, Vec<i64>), ApplicationError>
{
    let group_id = model_group::add_empty_group(group_name, &app_state.db).await;
    let mut model_ids = Vec::new();

    for entry in std::fs::read_dir(path)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.is_file())
        .filter(|f| match f.extension() {
            Some(ext) => ext == "stl" || ext == "obj" || ext == "3mf",
            None => false
        })
        {
            let file_name = util::prettify_file_name(&entry);
            let id = import_single_model(entry.to_str().unwrap(), &file_name, &app_state).await?;
            model_ids.push(id);
        }


    model_group::set_group_id_on_models(Some(group_id), model_ids.clone(), &app_state.db).await;

    Ok((group_id, model_ids))
}

// TODO: This should probably be refactored to use some kind of magic trait for file i/o, so i can skip the unzip on disk step.
pub async fn import_single_model(path : &str, name : &str, app_state : &AppState) -> Result<i64, ApplicationError> {
    let file_origin_path = PathBuf::from(path);
    let mut extension = String::from(file_origin_path.extension().unwrap().to_str().unwrap());

    let temp_file_path = PathBuf::from(&app_state.configuration.data_path)
        .join(format!("temp_model_file_{}.tmp", Utc::now().timestamp_nanos_opt().take().unwrap()));

    let is_stl = extension == "stl";
    
    let hash_string = match is_stl 
    {
        true => {
            let result = hash_and_zip_file(path, temp_file_path.to_str().unwrap(), &extension).await?;
            extension = String::from("stl.zip");
            result
        },
        false => hash_and_write_file(path, temp_file_path.to_str().unwrap()).await?
    };

    let model_dir = app_state.get_model_dir();
    let model_dir_path = PathBuf::from(model_dir);
    let to = model_dir_path.join(format!("{}.{}", hash_string, extension));

    let id = match model::get_model_id_via_sha256(&hash_string, &app_state.db).await 
    {
        Some(id) => 
        {
            fs::remove_file(temp_file_path).await?;
            id
        },
        None => 
        {
            fs::rename(temp_file_path, to).await?;
            model::add_model(name, &hash_string, &extension, &app_state.db).await
        }
    };
    
    return Ok(id);
}

async fn hash_and_write_file(path : &str, temp_file_path : &str) -> Result<String, ApplicationError>
{
    let mut src_file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buf: [u8; 32768] = [0u8; 32768];
    let mut temp_file = fs::File::create(&temp_file_path).await?;

    loop 
    { 
        let n = src_file.read(&mut buf).await?;

        if n == 0 
        { 
            break; 
        }

        hasher.update(&buf[..n]);
        temp_file.write_all(&buf[..n]).await?;
    }

    let hash_bytes = hasher.finalize();
    let hash_string = &format!("{:x}", hash_bytes)[0..32];

    return Ok(String::from(hash_string));
}

async fn hash_and_zip_file(path: &str, temp_file_path: &str, extension : &str) -> Result<String, ApplicationError> {
    let mut src_file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut file_contents = Vec::new();
    let mut buf = [0u8; 32768];
    
    loop 
    {
        let n = src_file.read(&mut buf).await?;
        
        if n == 0 
        { 
            break; 
        }

        hasher.update(&buf[..n]);
        file_contents.extend_from_slice(&buf[..n]);
    }

    let hash_bytes = hasher.finalize();
    let hash_string = &format!("{:x}", hash_bytes)[0..32];

    let temp_file_path_copy = String::from(temp_file_path);
    let filename = format!("{}.{}", hash_string, extension);

    spawn_blocking(move || -> Result<(), std::io::Error> {
        let file = std::fs::File::create(temp_file_path_copy)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(filename, options)?;
        zip.write_all(&file_contents)?;
        zip.finish()?;
        Ok(())
    }).await?.unwrap();

    return Ok(String::from(hash_string));
}

async fn import_single_model_2<W>(reader : W, file_type : &str, file_size : usize, name : &str, app_state : &AppState)// -> Result<i64, ApplicationError>
    where W: Read + Seek
{
    let mut file_contents = match file_size
    {
        0 => Vec::new(),
        val => Vec::with_capacity(val)
    };

    let read_byte_count = spawn_blocking(move || -> Result<usize, std::io::Error> {
        reader.read_to_end(&mut file_contents)
    }).await?;

    let hash = spawn_blocking(move || -> String {
        let mut hasher = Sha256::new();
        hasher.update(&file_contents);
        let bytes = hasher.finalize();
        String::from(&format!("{:x}", bytes)[0..32])
    }).await?;


}