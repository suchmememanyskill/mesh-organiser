use super::Slicer;
use crate::error::ApplicationError;
use crate::service::app_state::AppState;
use crate::db::model::Model;
use crate::service::slicer_service::open_custom_slicer;
use std::path::PathBuf;
use std::process::Command;
use crate::service::export_service::export_to_temp_folder;

impl Slicer {
    pub fn is_installed(&self) -> bool {
        if let Slicer::Custom = self
        {
            return true;
        }

        get_slicer_path(&self).is_some()
    }

    pub fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
        if let Slicer::Custom = self
        {
            return open_custom_slicer(models, app_state);
        }

        if !self.is_installed() {
            return Err(ApplicationError::InternalError(String::from(
                "Slicer not installed",
            )));
        }

        let (_, paths) = export_to_temp_folder(models, app_state, true, "open")?;

        println!("Opening in slicer: {:?}", paths);

        if paths.len() == 0 {
            return Err(ApplicationError::InternalError(String::from(
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
            let path = PathBuf::from("/Applications/Original Prusa Drivers/PrusaSlicer.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        },
        Slicer::OrcaSlicer => {
            let path = PathBuf::from("/Applications/OrcaSlicer.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        },
        Slicer::Cura => {
            let path = PathBuf::from("/Applications/UltiMaker Cura.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        },
        Slicer::BambuStudio => {
            let path = PathBuf::from("/Applications/BambuStudio.app");
            if path.exists() {
                return Some(path);
            }
            return None;
        },
        _ => None,
    }
}