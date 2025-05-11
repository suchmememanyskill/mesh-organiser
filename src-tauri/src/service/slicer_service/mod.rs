mod base;
use std::path::PathBuf;

pub use base::*;
use crate::{db::model::Model, error::ApplicationError, service::export_service::export_to_temp_folder};
use std::process::Command;

use super::app_state::AppState;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub fn open_with_paths(program: &str, paths: Vec<PathBuf>) -> Result<(), ApplicationError> {
    if paths.len() == 0 {
        return Err(ApplicationError::InternalError(String::from(
            "No models to open",
        )));
    }

    Command::new(program)
        .args(paths)
        .spawn()?;
    
    Ok(())
}

pub fn open_custom_slicer(models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError>
{
    let path = app_state.get_configuration().custom_slicer_path.clone();
    let pathbuf = PathBuf::from(path.clone());

    if path.is_empty() || !pathbuf.exists() {
        return Err(ApplicationError::InternalError(String::from(
            "Custom slicer path not set or is invalid",
        )));
    }

    let (_, paths) = export_to_temp_folder(models, app_state, true, "open")?;

    println!("Opening in slicer: {:?}", paths);
    open_with_paths(&path, paths)
}