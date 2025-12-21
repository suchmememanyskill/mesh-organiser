mod base;
use std::path::PathBuf;

use db::model::Model;

pub use base::*;
use std::process::Command;

use crate::{app_state, export_service::export_to_temp_folder, service_error::ServiceError};

use super::app_state::AppState;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub fn open_with_paths(program: &str, paths: Vec<PathBuf>) -> Result<(), ServiceError> {
    if paths.len() == 0 {
        return Err(ServiceError::InternalError(String::from(
            "No models to open",
        )));
    }

    Command::new(program).args(paths).spawn()?;

    Ok(())
}

pub async fn open_custom_slicer(
    paths: Vec<PathBuf>,
    app_state: &AppState,
) -> Result<(), ServiceError> {
    let path = app_state.get_configuration().custom_slicer_path.clone();
    let pathbuf = PathBuf::from(path.clone());

    if path.is_empty() || !pathbuf.exists() {
        return Err(ServiceError::InternalError(String::from(
            "Custom slicer path not set or is invalid",
        )));
    }

    println!("Opening in slicer: {:?}", paths);
    open_with_paths(&path, paths)
}
