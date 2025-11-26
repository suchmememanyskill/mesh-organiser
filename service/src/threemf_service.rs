use std::{path::PathBuf, thread};

use async_zip::tokio::read::seek::ZipFileReader;
use chrono::Utc;
use db::model::{Model, User};
use itertools::join;
use serde::{Deserialize, Serialize};
use stl_io::Vector;
use tokio::{fs::File, io::BufReader};

use crate::{AppState, ServiceError, cleanse_evil_from_name, export_service, import_service, import_state::ImportState};

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

    let temp_dir = std::env::temp_dir().join("meshorganiser_metadata_action");
    if !temp_dir.exists() {
        std::fs::create_dir(&temp_dir)?;
    }

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

fn extract_models_inner(theemf_path : PathBuf, temp_dir : &PathBuf) -> Result<(), ServiceError> {
    let handle = std::fs::File::open(theemf_path)?;
    let threemf_model = threemf::read(handle)?;

    let objects: Vec<_> = threemf_model
        .iter()
        .flat_map(|model| &model.resources.object)
        .filter(|obj| obj.mesh.is_some())
        .collect();

    for obj in objects {
        let mesh = obj.mesh.as_ref().unwrap();
        
        let obj_name = obj.name.as_ref()
            .map(|s| cleanse_evil_from_name(s))
            .unwrap_or_else(|| format!("object_{}", obj.id));
        
        let stl_path = temp_dir.join(format!("{}.stl", obj_name));

        let mut triangles = Vec::new();
        
        for triangle in &mesh.triangles.triangle {
            let v1 = &mesh.vertices.vertex[triangle.v1 as usize];
            let v2 = &mesh.vertices.vertex[triangle.v2 as usize];
            let v3 = &mesh.vertices.vertex[triangle.v3 as usize];
            
            let vertex1 = [v1.x as f32, v1.y as f32, v1.z as f32];
            let vertex2 = [v2.x as f32, v2.y as f32, v2.z as f32];
            let vertex3 = [v3.x as f32, v3.y as f32, v3.z as f32];
            
            let edge1 = [
                vertex2[0] - vertex1[0],
                vertex2[1] - vertex1[1],
                vertex2[2] - vertex1[2],
            ];
            let edge2 = [
                vertex3[0] - vertex1[0],
                vertex3[1] - vertex1[1],
                vertex3[2] - vertex1[2],
            ];
            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];
            
            triangles.push(stl_io::Triangle {
                normal: Vector(normal),
                vertices: [Vector(vertex1), Vector(vertex2), Vector(vertex3)],
            });
        }

        let mut file = std::fs::File::create(stl_path)?;
        stl_io::write_stl(&mut file, triangles.iter())?;
    }

    Ok(())
}

pub async fn extract_models(model : &Model, user: &User, app_state: &AppState) -> Result<ImportState, ServiceError> {
    if !model.blob.filetype.contains("3mf") {
        return Err(ServiceError::InternalError("Model is not a 3MF file".to_string()));
    }

    let mut temp_dir = std::env::temp_dir().join(format!(
        "meshorganiser_extract_action_{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&temp_dir)?;

    let theemf_path = export_service::get_path_from_model(&temp_dir, &model, app_state, true).await?;

    let safe_model_name = cleanse_evil_from_name(&model.name);
    temp_dir.push(safe_model_name);

    std::fs::create_dir(&temp_dir)?;
    
    {
        let temp_dir = temp_dir.clone();

        tokio::task::spawn_blocking(move || {
            extract_models_inner(theemf_path, &temp_dir)
        }).await??;
    }

    let import_state = ImportState::new(model.link.clone(), false, true, user.clone());
    let result = import_service::import_path(temp_dir.to_str().unwrap(), app_state, import_state).await?;

    Ok(result)
}