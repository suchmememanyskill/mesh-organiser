use std::env;
use std::fs::{self, File};
use std::io::Write;
use chrono::{format, Utc};
use serde::Serialize;
use urlencoding::decode;

use crate::error::ApplicationError;

#[derive(Serialize)]
pub struct DownloadResult
{
    pub path : String,
    pub source_uri : Option<String>,
}

pub async fn download_file(url: &str) -> Result<DownloadResult, ApplicationError> {
    let response = reqwest::get(url).await?;
    let mut source_uri : Option<String> = None;

    if !response.status().is_success() {
        return Err(ApplicationError::InternalError(format!("Failed to download file from url: {}. Status code {}.", url, response.status())));
    }
    
    let redirect_url_filename = match response.url().path_segments() {
        Some(segments) => String::from(decode(segments.last().unwrap_or("model.stl")).unwrap()),
        None => String::from("model.stl"),
    };

    let bytes = response.bytes().await?;
    
    let temp_dir = env::temp_dir().join(format!(
        "meshorganiser_download_action_{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    ));

    fs::create_dir_all(&temp_dir)?;

    let mut file_name = url.split('/').last().unwrap_or("model.stl");

    if url.contains("makerworld") {
        file_name = url.split("name=").last().unwrap_or("model.stl");
        source_uri = Some(String::from("https://www.thingiverse.com/"));
    }

    if url.contains("thingiverse") {
        file_name = &redirect_url_filename;
        source_uri = Some(String::from("https://makerworld.com"));
    }

    if url.starts_with("https://files.printables.com/media/prints/")
    {
        let id = String::from(url[42..].split("/").next().unwrap());
        source_uri = Some(format!("https://www.printables.com/model/{}", id));
    }

    let file_path = temp_dir.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(&bytes)?;

    Ok(DownloadResult {
        path : file_path.to_str().unwrap().to_string(),
        source_uri
    })
}