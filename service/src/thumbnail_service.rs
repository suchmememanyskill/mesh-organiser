use std::{panic, path::PathBuf};

use db::{blob_db, model::{self, Blob, Model}};
use image::imageops::FilterType::Triangle;
use libmeshthumbnail::{extract_image, parse_model, render};
use tokio::task::JoinSet;
use vek::{Vec2, Vec3};

use crate::{AppState, ServiceError, export_service::{get_image_path_for_blob, get_model_path_for_blob}, import_state::{ImportState, ImportStatus}};

const IMAGE_WIDTH: usize = 256;
const IMAGE_HEIGHT: usize = 256;
const IMAGE_SIZE: Vec2<usize> = Vec2::new(IMAGE_WIDTH, IMAGE_HEIGHT);

fn render(model_path: &PathBuf, image_path: &PathBuf, color: Vec3<u8>, rotation: Vec3<f32>) -> Result<(), ServiceError> {
    let mesh = match parse_model::handle_parse(model_path) {
        Ok(Some(mesh)) => mesh,
        Ok(None) => {
            return Err(ServiceError::InternalError(format!(
                "Could not generate thumbnail for model at path {:?}: unsupported format",
                model_path
            )));
        },
        Err(err) => {
            return Err(ServiceError::InternalError(format!(
                "Error parsing model at path {:?} for thumbnail generation: {}",
                model_path, err
            )));
        }
    };

    let thumbnail = render::render(
        &mesh, 
        IMAGE_SIZE, 
        rotation,
        color, 
        1.0);

    thumbnail.save(image_path)?;

    Ok(())
}

fn process(model_path: &PathBuf, image_path: &PathBuf, color: Vec3<u8>, rotation: Vec3<f32>, fallback_3mf_thumbnail : bool, prefer_3mf_thumbnail : bool, prefer_gcode_thumbnail : bool) -> Result<(), ServiceError> {
    let filename = model_path.to_string_lossy().to_lowercase();

    let extension = match filename {
        f if f.ends_with(".stl") => "stl",
        f if f.ends_with(".obj") => "obj",
        f if f.ends_with(".gcode") => "gcode",
        f if f.ends_with(".3mf") => "3mf",
        f if f.ends_with(".stl.zip") => "stl.zip",
        f if f.ends_with(".obj.zip") => "obj.zip",
        f if f.ends_with(".gcode.zip") => "gcode.zip",
        _ => {
            return Err(ServiceError::InternalError(format!(
                "Unsupported file extension for thumbnail generation: {:?}",
                model_path
            )));
        }
    };

    if (prefer_3mf_thumbnail && extension == "3mf")
        || (prefer_gcode_thumbnail && (extension == "gcode" || extension == "gcode.zip")) {
            if let Ok(Some(mut image)) = extract_image::handle_extract_image(&model_path) {
                image = image.resize_to_fill(IMAGE_WIDTH as u32, IMAGE_HEIGHT as u32, Triangle);
                image.save(&image_path)?;
                return Ok(());
            }
        }

    if render(model_path, image_path, color, rotation).is_ok() {
        return Ok(());
    }

    if fallback_3mf_thumbnail && !prefer_3mf_thumbnail && extension == "3mf" {
        if let Ok(Some(mut image)) = extract_image::handle_extract_image(&model_path) {
            image = image.resize_to_fill(IMAGE_WIDTH as u32, IMAGE_HEIGHT as u32, Triangle);
            image.save(&image_path)?;
            return Ok(());
        }
    }

    Err(ServiceError::InternalError(format!(
        "Failed to generate thumbnail for model at path {:?}",
        model_path
    )))
}

pub async fn generate_all_thumbnails(
    app_state: &AppState,
    overwrite: bool,
    import_state: &mut ImportState,
) -> Result<(), ServiceError> {
    let blobs = blob_db::get_blobs(&app_state.db).await?;
    let blob_refs: Vec<&Blob> = blobs.iter().collect();

    generate_thumbnails(&blob_refs, app_state, overwrite, import_state).await
}

pub async fn generate_thumbnails(
    models: &[&Blob],
    app_state: &AppState,
    overwrite: bool,
    import_state: &mut ImportState,
) -> Result<(), ServiceError> {
    import_state.update_status(ImportStatus::ProcessingThumbnails);
    let fallback_3mf_thumbnail = app_state.get_configuration().fallback_3mf_thumbnail;
    let prefer_3mf_thumbnail = app_state.get_configuration().prefer_3mf_thumbnail;
    let prefer_gcode_thumbnail = app_state.get_configuration().prefer_gcode_thumbnail;
    let max_concurrent = app_state.get_configuration().core_parallelism;
    let rotation_setting = app_state.get_configuration().thumbnail_rotation;
    let rotation = Vec3::new(
        rotation_setting[0] as f32,
        rotation_setting[1] as f32,
        rotation_setting[2] as f32,
    );

    let color = app_state
        .get_configuration()
        .thumbnail_color
        .replace("#", "")
        .to_uppercase();

    let color = u32::from_str_radix(&color, 16).unwrap_or(0xEEEEEE);

    let color = Vec3::new(
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
    );

    let paths: Vec<(PathBuf, PathBuf)> = models
        .iter()
        .map(|blob| {
            let model_path = get_model_path_for_blob(blob, app_state);
            let image_path = get_image_path_for_blob(blob, app_state);

            (model_path, image_path)
        })
        .filter(|(_, image_path)| {
            overwrite || !image_path.exists()
        })
        .collect();

    import_state.update_total_model_count(paths.len());

    let mut futures = JoinSet::new();
    let mut active = 0;

    for (model_path, image_path) in paths {
        let color = color.clone();
        let rotation = rotation.clone();
        futures.spawn_blocking(move || {
            // Ignore errors for now
            let _ = process(&model_path, &image_path, color, rotation, fallback_3mf_thumbnail, prefer_3mf_thumbnail, prefer_gcode_thumbnail);
        });
        active += 1;

        if active >= max_concurrent {
            if let Some(res) = futures.join_next().await {
                match res {
                    Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                    Err(err) => panic!("{err}"),
                    Ok(_) => {
                        active -= 1;
                        import_state.update_finished_thumbnails_count(1);
                    }
                }
            }
        }
    }

    futures.join_all().await;
    import_state.update_finished_thumbnails_count(import_state.model_count - import_state.finished_thumbnails_count);
    import_state.update_status(ImportStatus::FinishedThumbnails);
    Ok(())
}
