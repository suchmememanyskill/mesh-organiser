use std::env;
use std::fs::{self, File};
use std::io::Write;
use chrono::Utc;

use crate::error::ApplicationError;

pub async fn download_file(url: &str) -> Result<String, ApplicationError> {
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(ApplicationError::InternalError(format!("Failed to download file from url: {}. Status code {}.", url, response.status())));
    }
    
    let bytes = response.bytes().await?;
    
    let temp_dir = env::temp_dir().join(format!(
        "meshorganiser_download_action_{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    ));

    fs::create_dir_all(&temp_dir)?;

    let mut file_name = url.split('/').last().unwrap_or("downloaded_file");

    if url.contains("makerworld") {
        file_name = url.split("name=").last().unwrap_or("downloaded_file");
    }

    // TODO: Thingiverse is special for no reason

    let file_path = temp_dir.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(&bytes)?;

    Ok(file_path.to_str().unwrap().to_string())
}