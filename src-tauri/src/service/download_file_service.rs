use std::env;
use std::fs::{self, File};
use std::io::Write;
use chrono::{format, Utc};

use crate::error::ApplicationError;

pub async fn download_file(url: &str) -> Result<String, ApplicationError> {
    let response = reqwest::get(url).await?;
    let mut nonsense = String::from("");

    if !response.status().is_success() {
        return Err(ApplicationError::InternalError(format!("Failed to download file from url: {}. Status code {}.", url, response.status())));
    }
    
    let bytes = response.bytes().await?;
    
    let temp_dir = env::temp_dir().join(format!(
        "meshorganiser_download_action_{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    ));

    fs::create_dir_all(&temp_dir)?;

    let mut file_name = url.split('/').last().unwrap_or("model");

    if url.contains("makerworld") {
        file_name = url.split("name=").last().unwrap_or("model");
    }

    // TODO: Make this better
    if url.contains("thingiverse") {
        let id = url.split(":").last().unwrap_or("model");
        let extension;
        
        if bytes[0] == 0x50 && bytes[1] == 0x4B {
            extension = "3mf";
        } else {
            extension = "stl";
        }

        nonsense = format!("Thingiverse_{}.{}", id, extension);
        file_name = &nonsense;
    }

    let file_path = temp_dir.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(&bytes)?;

    Ok(file_path.to_str().unwrap().to_string())
}