use std::{env, panic, path::PathBuf};

use db::model::Model;
use tokio::{process::Command, task::JoinSet};

use crate::{error::ApplicationError, web_app_state::WebAppState};

pub async fn generate_thumbnails(
    models: &[Model],
    app_state: &WebAppState,
    overwrite: bool,
) -> Result<(), ApplicationError> {
    let image_path = app_state.get_image_dir();
    let model_path = app_state.get_model_dir();
    let fallback_3mf_thumbnail = app_state.get_configuration().fallback_3mf_thumbnail;
    let prefer_3mf_thumbnail = app_state.get_configuration().prefer_3mf_thumbnail;
    let prefer_gcode_thumbnail = app_state.get_configuration().prefer_gcode_thumbnail;
    let max_concurrent = app_state.get_configuration().core_parallelism;
    let mesh_thumbnail_executable_path = match env::var("MESH_THUMBNAIL_EXECUTABLE_PATH") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            return Err(ApplicationError::InternalError(
                "MESH_THUMBNAIL_EXECUTABLE_PATH environment variable not set".to_string(),
            ));
        }
    };

    if !mesh_thumbnail_executable_path.exists() {
        return Err(ApplicationError::InternalError(format!(
            "Mesh thumbnail executable not found at path: {:?}",
            mesh_thumbnail_executable_path
        )));
    }

    let color = app_state
        .get_configuration()
        .thumbnail_color
        .replace("#", "")
        .to_uppercase();

    let paths: Vec<String> = models
        .iter()
        .map(|f| {
            let new_path = model_path.join(format!("{}.{}", f.blob.sha256, f.blob.filetype));
            let text_path = new_path.to_str().unwrap().to_string();

            text_path
        })
        .collect();

    let mut futures = JoinSet::new();
    let mut active = 0;
    let total = paths.len() / 100 + 1;

    for (i, chunk) in paths.chunks(100).enumerate() {
        let mut command = Command::new(&mesh_thumbnail_executable_path);
        command
            .arg("--rotatey")
            .arg("25")
            .arg("--format")
            .arg("png")
            .arg("--outdir")
            .arg(image_path.to_str().unwrap())
            .arg("--color")
            .arg(&color);

        if fallback_3mf_thumbnail {
            command.arg("--fallback-3mf-thumbnail");
        }

        if fallback_3mf_thumbnail && prefer_3mf_thumbnail {
            command.arg("--prefer-3mf-thumbnail");
        }

        if prefer_gcode_thumbnail {
            command.arg("--prefer-gcode-thumbnail");
        }

        if overwrite {
            command.arg("--overwrite");
        }

        command.args(chunk);
        
        println!("About to spawn: {:?}", mesh_thumbnail_executable_path);
        println!("{:?}", command);
        let spawned_command = command.spawn()?;
        active += 1;

        // TODO: Replace with trace
        println!("Started thumbnail generation command {}/{}", i + 1, total);
        futures.spawn(async move {
            let mut spawned_command = spawned_command;
            spawned_command.wait().await
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
    }

    futures.join_all().await;
    // TODO: Replace with trace
    println!("Completed thumbnail generation");

    Ok(())
}
