use std::{path::PathBuf, time::Duration};

use tauri::AppHandle;
use tokio::sync::mpsc::error::TryRecvError;

use crate::{
    error::ApplicationError,
    service::import_state::{ImportState, ImportStatus},
};

use super::app_state::AppState;
use crate::db::model::{self, Model};

use tauri_plugin_shell::{process::Command, ShellExt};

pub async fn generate_all_thumbnails(
    app_state: &AppState,
    app_handle: &AppHandle,
    overwrite: bool,
) -> Result<(), ApplicationError> {
    let models = model::get_models(&app_state.db).await;

    let mut import_state = &mut ImportState::new(None, false, false);

    generate_thumbnails(&models, app_state, app_handle, overwrite, &mut import_state).await?;

    Ok(())
}

pub async fn generate_thumbnails(
    models: &Vec<Model>,
    app_state: &AppState,
    app_handle: &AppHandle,
    overwrite: bool,
    import_state: &mut ImportState,
) -> Result<(), ApplicationError> {
    import_state.update_status(ImportStatus::ProcessingThumbnails, app_handle);
    let image_path = PathBuf::from(app_state.get_image_dir());
    let model_path = PathBuf::from(app_state.get_model_dir());
    let fallback_3mf_thumbnail = app_state.get_configuration().fallback_3mf_thumbnail;
    let prefer_3mf_thumbnail = app_state.get_configuration().prefer_3mf_thumbnail;
    let prefer_gcode_thumbnail = app_state.get_configuration().prefer_gcode_thumbnail;
    let max_concurrent = app_state.get_configuration().core_parallelism;

    let color = app_state
        .get_configuration()
        .thumbnail_color
        .replace("#", "")
        .to_uppercase();

    let paths: Vec<String> = models
        .iter()
        .map(|f| {
            let new_path = model_path.join(format!("{}.{}", f.sha256, f.filetype));
            let text_path = new_path.to_str().unwrap().to_string();

            text_path
        })
        .collect();

    struct C {
        command: Command,
        thumbnail_count: usize,
    }

    struct D {
        thumbnail_count: usize,
        listener: tauri::async_runtime::Receiver<tauri_plugin_shell::process::CommandEvent>,
        child: tauri_plugin_shell::process::CommandChild,
    }

    let mut commands: Vec<C> = paths
        .chunks(100)
        .map(|slice| {
            let len = slice.len();

            let mut command = app_handle.shell().sidecar("mesh-thumbnail").unwrap();

            command = command
                .arg("--rotatey")
                .arg("25")
                .arg("--format")
                .arg("png")
                .arg("--outdir")
                .arg(image_path.to_str().unwrap())
                .arg("--color")
                .arg(&color);

            if fallback_3mf_thumbnail {
                command = command.arg("--fallback-3mf-thumbnail");
            }

            if fallback_3mf_thumbnail && prefer_3mf_thumbnail {
                command = command.arg("--prefer-3mf-thumbnail");
            }

            if prefer_gcode_thumbnail {
                command = command.arg("--prefer-gcode-thumbnail");
            }

            if overwrite {
                command = command.arg("--overwrite");
            }

            command = command.args(slice);

            //println!("{:?}", command);

            C {
                command,
                thumbnail_count: len,
            }
        })
        .collect();
    /*
        #[cfg(debug_assertions)]
        {
            while !commands.is_empty() {
                let command_wrapper = commands.pop().unwrap();
                let result = command_wrapper.command.output().await;
                match result {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            println!("Error: {}", stderr);
                        }
                    }
                    Err(e) => {
                        println!("Failed to execute command: {}", e);
                    }
                }
            }

            return Ok(());
        }
    */

    let mut running = Vec::new();

    println!("Using {} threads for thumbnail generation", max_concurrent);

    while !commands.is_empty() || !running.is_empty() {
        if !commands.is_empty() && running.len() < max_concurrent {
            let command = commands.pop().unwrap();
            let a = command.command.spawn().expect("Failed to spawn command");

            running.push(D {
                thumbnail_count: command.thumbnail_count,
                listener: a.0,
                child: a.1,
            });
        } else {
            let mut i = 0;
            while i < running.len() {
                let run = &mut running[i];

                let res = run.listener.try_recv();

                if let Err(e) = res {
                    if e == TryRecvError::Disconnected {
                        import_state
                            .update_finished_thumbnails_count(run.thumbnail_count, app_handle);
                        running.remove(i);
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    import_state.update_status(ImportStatus::FinishedThumbnails, app_handle);

    Ok(())
}
