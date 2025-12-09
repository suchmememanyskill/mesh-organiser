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

        let package = get_flatpak_slicer_package(&self);

        if package.is_empty() {
            return false;
        }

        match Command::new("flatpak").arg("info").arg(package).output() {
            Ok(output) => {
                return output.status.success();
            }
            Err(_) => {
                return false;
            }
        }
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

        let _ = Command::new("flatpak")
            .arg("run")
            .arg("--file-forwarding")
            .arg(get_flatpak_slicer_package(&self))
            .arg("@@")
            .args(paths)
            .arg("@@")
            .spawn()?;

        Ok(())
    }
}

fn get_flatpak_slicer_package(slicer: &Slicer) -> String {
    match slicer {
        Slicer::PrusaSlicer => "com.prusa3d.PrusaSlicer",
        Slicer::OrcaSlicer => "io.github.softfever.OrcaSlicer",
        Slicer::Cura => "com.ultimaker.cura",
        Slicer::BambuStudio => "com.bambulab.BambuStudio",
        _ => "",
    }
    .to_string()
}
