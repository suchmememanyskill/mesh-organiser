use std::path::PathBuf;

use crate::db::model::Model;
use crate::service::export_service::export_to_temp_folder;

use super::app_state::AppState;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use strum::EnumIter;
#[cfg(target_os = "windows")]
use winreg::*;

// TODO: Make all of this async

#[derive(Clone, Serialize, Deserialize, EnumIter)]
pub enum Slicer {
    PrusaSlicer,
    OrcaSlicer,
    Cura,
    BambuStudio,
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
                return get_cura_path().is_some();
            }
            _ => {
                return false;
            }
        }
    }

    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "windows")]
    pub fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
        if !self.is_installed() {
            return Err(ApplicationError::InternalError(String::from(
                "Slicer not installed",
            )));
        }

        let slicer_path = match *self {
            Slicer::PrusaSlicer => get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Prusa3D\\PrusaSlicer\\Settings",
                "InstallPath",
            ),
            Slicer::BambuStudio => match get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Bambulab\\Bambu Studio",
                "InstallPath",
            ) {
                Some(s) => Some(String::from(
                    PathBuf::from(s).join("bambu-studio.exe").to_str().unwrap(),
                )),
                None => None,
            },
            Slicer::OrcaSlicer => match get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\WOW6432Node\\SoftFever\\OrcaSlicer",
                "",
            ) {
                Some(s) => Some(String::from(
                    PathBuf::from(s).join("orca-slicer.exe").to_str().unwrap(),
                )),
                None => None,
            },
            Slicer::Cura => Some(String::from(
                get_cura_path().take().unwrap().to_str().unwrap(),
            )),
            _ => {
                return Err(ApplicationError::InternalError(String::from(
                    "Slicer not installed",
                )));
            }
        }
        .take()
        .unwrap();

        let (_, paths) = export_to_temp_folder(models, app_state, true, "open")?;

        println!("Opening in slicer: {:?}", paths);

        if paths.len() == 0 {
            return Err(ApplicationError::InternalError(String::from(
                "No models to open",
            )));
        }

        Command::new(slicer_path)
            .args(paths)
            .output()
            .expect("failed to execute process");

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
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
            .arg(get_flatpak_slicer_package(&self))
            .arg("--")
            .args(paths)
            .output()?;

        Ok(())
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

#[cfg(target_os = "windows")]
fn get_cura_path() -> Option<PathBuf> {
    let program_files = "C:\\Program Files";
    if let Ok(entries) = fs::read_dir(program_files) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(folder_name) = entry.file_name().to_str() {
                    if folder_name.starts_with("UltiMaker Cura") {
                        let exe_path = Path::new(program_files)
                            .join(folder_name)
                            .join("UltiMaker-Cura.exe");
                        if exe_path.exists() {
                            return Some(exe_path);
                        }
                    }
                }
            }
        }
    }

    None
}
