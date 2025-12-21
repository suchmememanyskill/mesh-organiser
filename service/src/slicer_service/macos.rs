use super::Slicer;
use db::model::Model;
use crate::service_error::ServiceError;
use crate::app_state::AppState;
use crate::export_service::export_to_temp_folder;
use crate::slicer_service::open_custom_slicer;
use std::path::PathBuf;
use std::process::Command;

impl Slicer {
    pub fn is_installed(&self) -> bool {
        if let Slicer::Custom = self {
            return true;
        }

        get_slicer_path(&self).is_some()
    }

    pub async fn open(&self, paths: Vec<PathBuf>, app_state: &AppState) -> Result<(), ServiceError> {
        if let Slicer::Custom = self {
            return open_custom_slicer(paths, app_state).await;
        }

        if !self.is_installed() {
            return Err(ServiceError::InternalError(String::from(
                "Slicer not installed",
            )));
        }

        println!("Opening in slicer: {:?}", paths);

        if paths.len() == 0 {
            return Err(ServiceError::InternalError(String::from(
                "No models to open",
            )));
        }

        let slicer_path = get_slicer_path(&self).unwrap();

        Command::new("open")
            .arg("-a")
            .arg(slicer_path)
            .arg("--args")
            .args(paths)
            .spawn()?;

        Ok(())
    }
}

fn get_slicer_path(slicer: &Slicer) -> Option<PathBuf> {
    match slicer {
        Slicer::PrusaSlicer => {
            let path: PathBuf =
                PathBuf::from("/Applications/Original Prusa Drivers/PrusaSlicer.app");
            let second_path = PathBuf::from("/Applications/PrusaSlicer.app");

            if path.exists() {
                return Some(path);
            }

            if second_path.exists() {
                return Some(second_path);
            }

            return None;
        }
        Slicer::OrcaSlicer => {
            let path = PathBuf::from("/Applications/OrcaSlicer.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        }
        Slicer::Cura => {
            let path = PathBuf::from("/Applications/UltiMaker Cura.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        }
        Slicer::BambuStudio => {
            let path = PathBuf::from("/Applications/BambuStudio.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        }
        _ => None,
    }
}
