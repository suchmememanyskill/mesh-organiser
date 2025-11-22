use async_zip::tokio::read::seek::ZipFileReader;
use chrono::Utc;
use db::model::Model;
use itertools::join;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::BufReader};

use crate::{AppState, ServiceError, export_service};

#[derive(Deserialize)]
pub struct ProjectSettingsConfig {
    pub nozzle_diameter: Vec<String>,
    pub layer_height: String,
    pub filament_type: Vec<String>,
    pub enable_support: String,
}

#[derive(Serialize)]
pub struct ThreemfMetadata {
    pub nozzle_diameter: Option<f32>,
    pub layer_height: Option<f32>,
    pub material_type: Option<String>,
    pub supports_enabled: Option<bool>,
}

async fn parse_project_settings_config(data: String) -> Result<ThreemfMetadata, ServiceError> {
    let parsed_data = serde_json::from_str::<ProjectSettingsConfig>(&data)?;

    let nozzle_diameter = parsed_data.nozzle_diameter.get(0)
        .and_then(|s| s.parse::<f32>().ok());

    let layer_height = parsed_data.layer_height.parse::<f32>().ok();

    let filament_type = join(parsed_data.filament_type.into_iter(), ", ");

    let enable_support = match parsed_data.enable_support.as_str() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    };

    Ok(ThreemfMetadata {
        nozzle_diameter,
        layer_height,
        material_type: Some(filament_type),
        supports_enabled: enable_support,
    })
}

async fn parse_slicer_pe_config(data: String) -> Result<ThreemfMetadata, ServiceError> {
    let mut threemf = ThreemfMetadata {
        nozzle_diameter: None,
        layer_height: None,
        material_type: None,
        supports_enabled: None,
    };

    match data.lines().find(|line| line.starts_with("; nozzle_diameter = ")) {
        Some(line) => {
            let value = line.trim_start_matches("; nozzle_diameter = ").trim();
            let nozzle_diameters = value.split(',').collect::<Vec<&str>>();

            threemf.nozzle_diameter = nozzle_diameters.get(0)
                .and_then(|s| s.parse::<f32>().ok());
        },
        None => {}
    }

    match data.lines().find(|line| line.starts_with("; layer_height = ")) {
        Some(line) => {
            let value = line.trim_start_matches("; layer_height = ").trim();

            threemf.layer_height = value.parse::<f32>().ok();
        },
        None => {}
    }

    match data.lines().find(|line| line.starts_with("; filament_type = ")) {
        Some(line) => {
            let value = line.trim_start_matches("; filament_type = ").trim();
            threemf.material_type = Some(value.replace(";", ", "));
        },
        None => {}
    }

    match data.lines().find(|line| line.starts_with("; support_material =")) {
        Some(line) => {
            let value = line.trim_start_matches("; support_material =").trim();
            threemf.supports_enabled = match value {
                "1" => Some(true),
                "0" => Some(false),
                _ => None,
            };
        },
        None => {}
    }

    Ok(threemf)
}

pub async fn extract_metadata(model : &Model, app_state: &AppState) -> Result<ThreemfMetadata, ServiceError> {
    if !model.blob.filetype.contains("3mf") {
        return Err(ServiceError::InternalError("Model is not a 3MF file".to_string()));
    }

    let temp_dir = std::env::temp_dir().join(format!(
        "meshorganiser_metadata_action_{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&temp_dir)?;

    let theemf_path = export_service::get_path_from_model(&temp_dir, &model, app_state, true).await?;
    
    let zip_file = File::open(theemf_path).await?;
    let mut buffered_reader = BufReader::new(zip_file);
    let mut zip = ZipFileReader::with_tokio(&mut buffered_reader).await?;

    let entries : Vec<_> = zip.file().entries().iter().cloned().collect();

    for (i, entry) in entries.iter().enumerate() {
        let entry_filename = match entry.filename().as_str() {
            Ok(name) => name,
            Err(_) => continue,
        };

        if entry_filename.ends_with("project_settings.config") {
            let mut file = zip.reader_with_entry(i).await?;
            let mut contents = String::new();
            file.read_to_string_checked(&mut contents).await?;
            return parse_project_settings_config(contents).await;
        }

        if entry_filename.ends_with("Slic3r_PE.config") {
            let mut file = zip.reader_with_entry(i).await?;
            let mut contents = String::new();
            file.read_to_string_checked(&mut contents).await?;
            return parse_slicer_pe_config(contents).await;
        }
    }

    return Err(ServiceError::InternalError("Failed to extract metadata".to_string()));
}