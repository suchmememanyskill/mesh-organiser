use super::Slicer;
use crate::error::ApplicationError;
use crate::service::app_state::AppState;
use crate::db::model::Model;
use std::process::Command;
use crate::service::export_service::export_to_temp_folder;

impl Slicer 
{
    pub fn is_installed(&self) -> bool {
        match Command::new("flatpak")
        .arg("info")
        .arg(get_flatpak_slicer_package(&self))
        .output()
        {
            Ok(output) => {
                return output.status.success();
            }
            Err(_) => {
                return false;
            }
        }
    }

    pub fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
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

fn get_flatpak_slicer_package(slicer : &Slicer) -> String
{
    match slicer {
        Slicer::PrusaSlicer => "com.prusa3d.PrusaSlicer",
        Slicer::OrcaSlicer => "io.github.softfever.OrcaSlicer",
        Slicer::Cura => "com.ultimaker.cura",
        Slicer::BambuStudio => "com.bambulab.BambuStudio",
    }.to_string()
}