use chrono::Utc;
use regex::Regex;
use reqwest::Response;
use reqwest::header::CONTENT_DISPOSITION;
use serde::Serialize;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use urlencoding::decode;

use crate::service_error::ServiceError;
use crate::util::cleanse_evil_from_name;

#[derive(Serialize)]
pub struct DownloadResult {
    pub path: String,
    pub source_uri: Option<String>,
}

fn get_content_disposition_filename(response : &Response) -> Option<String> {
    let content_disposition = response.headers().get(CONTENT_DISPOSITION);

    match content_disposition {
        None => match response.url().path().split("/").last() {
            Some(x) => Some(String::from(x)),
            None => return None,
        },
        Some(header_value) => match header_value.to_str() {
            Ok(header_value) => match content_disposition::parse_content_disposition(header_value).filename_full() {
                Some(filename) => Some(filename),
                None => return None
            }
            Err(_) => return None
        }
    }
}

pub async fn download_file(url: &str) -> Result<DownloadResult, ServiceError> {
    let response = reqwest::get(url).await?;
    let mut source_uri: Option<String> = None;

    if !response.status().is_success() {
        return Err(ServiceError::InternalError(format!(
            "Failed to download file from url: {}. Status code {}.",
            url,
            response.status()
        )));
    }

    let redirect_url_filename = match response.url().path_segments() {
        Some(segments) => String::from(decode(segments.last().unwrap_or("model.stl")).unwrap()),
        None => String::from("model.stl"),
    };

    

    let temp_dir = env::temp_dir().join(format!(
        "meshorganiser_download_action_{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    ));

    fs::create_dir_all(&temp_dir)?;

    let mut file_name = url.split('/').last().unwrap_or("model.stl");
    let mut tmp: String;

    if url.contains("makerworld") {
        file_name = url.split("name=").last().unwrap_or("model.stl");
        source_uri = Some(String::from("https://makerworld.com"));
    } else if url.contains("thingiverse") {
        file_name = &redirect_url_filename;
        source_uri = Some(String::from("https://www.thingiverse.com/"));
    } else if url.starts_with("https://files.printables.com/media/prints/") {
        let id = String::from(url[42..].split("/").next().unwrap());
        source_uri = Some(format!("https://www.printables.com/model/{}", id));
    } else if url.contains("nexprint") {
        let re = Regex::new(r#"filename="([^"]+)""#).unwrap();
        let decoded_url = decode(url).unwrap().into_owned();
        if let Some(caps) = re.captures(&decoded_url) {
            if let Some(m) = caps.get(1) {
                tmp = String::from(m.as_str());
                file_name = tmp.as_str();
            }
        }

        source_uri = Some(String::from("https://www.nexprint.com/"));
    } else {
        if let Some(filename) = get_content_disposition_filename(&response) {
            tmp = filename;
            file_name = &tmp;
        }
    }

    tmp = cleanse_evil_from_name(file_name);
    file_name = &tmp;
    let file_path = temp_dir.join(file_name);
    let mut file = File::create(&file_path)?;
    let bytes = response.bytes().await?;
    file.write_all(&bytes)?;

    Ok(DownloadResult {
        path: file_path.to_str().unwrap().to_string(),
        source_uri,
    })
}
