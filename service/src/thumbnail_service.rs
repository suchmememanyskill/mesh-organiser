use std::{panic, path::PathBuf};

use db::model::{self, Model};
use image::imageops::FilterType::Triangle;
use libmeshthumbnail::{extract_image, parse_model, render};
use tokio::task::JoinSet;
use vek::{Vec2, Vec3};

use crate::{AppState, ServiceError, import_state::{ImportState, ImportStatus}};

const IMAGE_WIDTH: usize = 512;
const IMAGE_HEIGHT: usize = 512;
const FIXED_ANGLE: Vec3<f32> = Vec3::new(35.0, 30.0, 0.0);
const IMAGE_SIZE: Vec2<usize> = Vec2::new(IMAGE_WIDTH, IMAGE_HEIGHT);

fn render(model_path: &PathBuf, image_path: &PathBuf, color: Vec3<u8>) -> Result<(), ServiceError> {
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
        FIXED_ANGLE, // TODO: Maybe make configurable?
        color, 
        1.0);

    thumbnail.save(image_path)?;

    Ok(())
}

fn process(model_path: &PathBuf, image_path: &PathBuf, color: Vec3<u8>, fallback_3mf_thumbnail : bool, prefer_3mf_thumbnail : bool, prefer_gcode_thumbnail : bool) -> Result<(), ServiceError> {
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

    if render(model_path, image_path, color).is_ok() {
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

pub async fn generate_thumbnails(
    models: &[Model],
    app_state: &AppState,
    overwrite: bool,
    import_state: &mut ImportState,
) -> Result<(), ServiceError> {
    import_state.update_status(ImportStatus::ProcessingThumbnails);
    let image_path = app_state.get_image_dir();
    let model_path = app_state.get_model_dir();
    let fallback_3mf_thumbnail = app_state.get_configuration().fallback_3mf_thumbnail;
    let prefer_3mf_thumbnail = app_state.get_configuration().prefer_3mf_thumbnail;
    let prefer_gcode_thumbnail = app_state.get_configuration().prefer_gcode_thumbnail;
    let max_concurrent = app_state.get_configuration().core_parallelism;

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
        .map(|f| {
            let model_path = model_path.join(format!("{}.{}", f.blob.sha256, f.blob.filetype));
            let image_path = image_path.join(format!("{}.png", f.blob.sha256));

            (model_path, image_path)
        })
        .filter(|(_, image_path)| {
            overwrite || !image_path.exists()
        })
        .collect();

    import_state.update_total_model_count(paths.len());

    let mut futures = JoinSet::new();
    let mut active = 0;
    let total = paths.len() / 100 + 1;

    for (model_path, image_path) in paths {
        let color = color.clone();
        futures.spawn_blocking(move || {
            // Ignore errors for now
            let _ = process(&model_path, &image_path, color, fallback_3mf_thumbnail, prefer_3mf_thumbnail, prefer_gcode_thumbnail);
        });

        if active >= max_concurrent {
            if let Some(res) = futures.join_next().await {
                match res {
                    Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                    Err(err) => panic!("{err}"),
                    Ok(_) => {
                        active -= 1;
                    }
                }
            }
        }

        import_state.update_finished_thumbnails_count(1);
    }

    futures.join_all().await;
    import_state.update_finished_thumbnails_count(total);
    import_state.update_status(ImportStatus::FinishedThumbnails);
    Ok(())
}
