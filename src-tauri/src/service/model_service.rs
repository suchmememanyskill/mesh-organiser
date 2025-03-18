use crate::{db::model, error::ApplicationError};
use super::app_state::AppState;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use tokio::{self, fs};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use chrono::Utc;
use tokio::task::spawn_blocking;
use zip;
use std::io::Write;
use zip::write::SimpleFileOptions;

pub async fn import_model(path : &str, name : &str, app_state : AppState) -> Result<(), ApplicationError> {
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

    fs::rename(temp_file_path, to).await?;

    model::add_model(name, &hash_string, &extension, &app_state.db).await;
    
    return Ok(());
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
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Lzma);
        zip.start_file(filename, options)?;
        zip.write_all(&file_contents)?;
        zip.finish()?;
        Ok(())
    }).await?.unwrap();

    return Ok(String::from(hash_string));
}