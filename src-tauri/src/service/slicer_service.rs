use std::{fs::File, path::PathBuf};

use crate::db::model::Model;

use super::app_state::AppState;
use crate::error::ApplicationError;
use chrono::Utc;
use std::process::Command;
#[cfg(target_os = "windows")]
use winreg::*;

// TODO: Make all of this async

pub enum Slicer {
    PrusaSlicer,
    OrcaSlicer,
    Cura,
    BambuStudio,
}

impl Slicer {
    #[cfg(target_os = "windows")]
    pub fn is_installed(&self) -> bool {
        match *self {
            Slicer::PrusaSlicer => {
                return get_registry_key(
                    winreg::enums::HKEY_LOCAL_MACHINE,
                    "SOFTWARE\\Prusa3D\\PrusaSlicer\\Settings",
                    "InstallPath",
                )
                .is_some();
            }
            Slicer::BambuStudio => {
                return get_registry_key(
                    winreg::enums::HKEY_LOCAL_MACHINE,
                    "SOFTWARE\\Bambulab\\Bambu Studio",
                    "InstallPath",
                )
                .is_some();
            }
            Slicer::OrcaSlicer => {
                return get_registry_key(
                    winreg::enums::HKEY_LOCAL_MACHINE,
                    "SOFTWARE\\WOW6432Node\\SoftFever\\OrcaSlicer",
                    "",
                )
                .is_some();
            }
            Slicer::Cura => {
                return false; // TODO
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub fn is_installed(&self) -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    pub fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
        if !self.is_installed() {
            return Err(ApplicationError::InternalError);
        }

        let slicer_path = match *self {
            Slicer::PrusaSlicer => get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Prusa3D\\PrusaSlicer\\Settings",
                "InstallPath",
            ),
            Slicer::BambuStudio => get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Bambulab\\Bambu Studio",
                "InstallPath",
            ),
            Slicer::OrcaSlicer => get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\WOW6432Node\\SoftFever\\OrcaSlicer",
                "",
            ),
            Slicer::Cura => {
                None // TODO
            }
        }
        .take()
        .unwrap();

        let temp_dir = std::env::temp_dir().join(format!(
            "meshorganiser_open_action_{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir(&temp_dir)?;

        let paths: Vec<PathBuf> = models
            .iter()
            .map(|f| get_path_from_model(&temp_dir, f, &app_state).unwrap())
            .collect();

        let mut command = Command::new(slicer_path)
            .args(paths)
            .output()
            .expect("failed to execute process");

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
        Err(ApplicationError::InternalError)
    }
}

#[cfg(target_os = "windows")]
fn get_registry_key(root: HKEY, subkey: &str, field: &str) -> Option<String> {
    use std::ffi::OsString;

    let reg_key_result = winreg::RegKey::predef(root).open_subkey(subkey);

    if reg_key_result.is_err() {
        return None;
    }

    let reg_key = reg_key_result.unwrap();

    let value: Result<OsString, std::io::Error> = reg_key.get_value(field);

    match value {
        Ok(s) => return Some(s.to_str().unwrap().to_string()),
        Err(_) => return None,
    }
}

fn get_path_from_model(
    temp_dir: &PathBuf,
    model: &Model,
    app_state: &AppState,
) -> Result<PathBuf, ApplicationError> {
    let base_dir = PathBuf::from(app_state.get_model_dir());

    if model.filetype == "stl.zip" {
        let zip_path = base_dir.join(format!("{}.stl.zip", model.sha256));
        let file = File::open(zip_path)?;

        let target = temp_dir.join(format!("{}.stl", model.sha256));
        let mut archive = zip::ZipArchive::new(file)?;
        let mut file = archive.by_index(0)?;
        let mut target_file = File::create(&target)?;

        std::io::copy(&mut file, &mut target_file)?;
        Ok(target)
    } else {
        let file_path = base_dir.join(format!("{}.{}", model.sha256, model.filetype));
        Ok(file_path)
    }
}
