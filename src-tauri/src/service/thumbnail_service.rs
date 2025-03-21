use std::path::PathBuf;

use tauri::AppHandle;

use crate::error::ApplicationError;

use super::app_state::AppState;
use crate::db::model::{self, Model};

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

pub async fn generate_all_thumbnails(app_state: &AppState, app_handle : &AppHandle, overwrite : bool) -> Result<(), ApplicationError>
{
    let models = model::get_models(&app_state.db).await;

    generate_thumbnails(models, app_state, app_handle, overwrite).await?;

    Ok(())
}

pub async fn generate_thumbnails(models: Vec<Model>, app_state: &AppState, app_handle : &AppHandle, overwrite : bool) -> Result<(), ApplicationError>
{
    let image_path = PathBuf::from(app_state.get_image_dir());
    let model_path = PathBuf::from(app_state.get_model_dir());

    let paths : Vec<String> = models
        .iter()
        .map(|f| {
            let new_path = model_path.join(format!("{}.{}", f.sha256, f.filetype));
            let text_path = new_path.to_str().unwrap().to_string();

            text_path
        })
        .collect();

    for paths_slice in paths.chunks(100)
    {
        let mut command = app_handle.shell().sidecar("mesh-thumbnail").unwrap();

        command = command
            .arg("--rotatey").arg("25")
            .arg("--format").arg("png")
            .arg("--outdir").arg(image_path.to_str().unwrap());
    
        if overwrite {
            command = command.arg("--overwrite");
        }
    
        command = command.args(paths_slice);
        let output = command.output().await?;
    }
    
    /*
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    println!("Thumbnail generation output: {}", stdout);
    println!("Thumbnail generation error: {}", stderr);
    */

    Ok(())
}