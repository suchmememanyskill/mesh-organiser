use super::{Slicer, open_custom_slicer};
use db::model::Model;
use crate::error::ApplicationError;
use crate::service::app_state::AppState;
use crate::service::export_service::export_to_temp_folder;
use crate::service::slicer_service::open_with_paths;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use winreg::*;

impl Slicer {
    pub fn is_installed(&self) -> bool {
        if let Slicer::Custom = self {
            return true;
        }

        get_slicer_path(&self).is_some()
    }

    pub fn open(&self, models: Vec<Model>, app_state: &AppState) -> Result<(), ApplicationError> {
        if let Slicer::Custom = self {
            return open_custom_slicer(models, app_state);
        }

        if !self.is_installed() {
            return Err(ApplicationError::InternalError(String::from(
                "Slicer not installed",
            )));
        }

        let slicer_pathbuf = get_slicer_path(&self).unwrap();
        let slicer_path = slicer_pathbuf.to_str().unwrap();
        let (_, paths) = export_to_temp_folder(models, app_state, true, "open")?;

        println!("Opening in slicer: {:?}", paths);

        open_with_paths(slicer_path, paths)
    }
}

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

fn get_slicer_path(slicer: &Slicer) -> Option<PathBuf> {
    match slicer {
        Slicer::PrusaSlicer => {
            let key = get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Prusa3D\\PrusaSlicer\\Settings",
                "InstallPath",
            );

            if let Some(key) = key {
                let path = PathBuf::from(key);

                if path.exists() {
                    return Some(path);
                }
            }

            let path = PathBuf::from("C:\\Program Files\\Prusa3D\\PrusaSlicer\\prusa-slicer.exe");

            if path.exists() {
                return Some(path);
            }

            return None;
        }
        Slicer::BambuStudio => {
            if let Some(key) = get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Bambulab\\Bambu Studio",
                "InstallPath",
            ) {
                let path = PathBuf::from(key).join("bambu-studio.exe");

                if path.exists() {
                    return Some(path);
                }
            }

            let path = PathBuf::from("C:\\Program Files\\Bambu Studio\\bambu-studio.exe");

            if path.exists() {
                return Some(path);
            }

            return None;
        }
        Slicer::OrcaSlicer => {
            if let Some(key) = get_registry_key(
                winreg::enums::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\WOW6432Node\\SoftFever\\OrcaSlicer",
                "",
            ) {
                let path = PathBuf::from(key).join("orca-slicer.exe");

                if path.exists() {
                    return Some(path);
                }
            }

            let path = PathBuf::from("C:\\Program Files\\OrcaSlicer\\orca-slicer.exe");

            if path.exists() {
                return Some(path);
            }

            return None;
        }
        Slicer::Cura => {
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

            return None;
        }
        _ => None,
    }
}
