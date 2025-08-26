use regex::Regex;
use std::ffi::OsStr;
use std::process::Command;
use std::{io::Read, path::PathBuf};

use crate::error::ApplicationError;

pub fn prettify_file_name(file: &PathBuf, is_dir: bool) -> String {
    let extension = file.extension();
    let mut file_name: String = String::from(
        file.file_name()
            .unwrap_or(OsStr::new("unknown_filename"))
            .to_str()
            .unwrap(),
    );

    if !is_dir {
        match extension {
            Some(ext) => {
                file_name = String::from(&file_name[0..file_name.len() - ext.len() - 1]);
            }
            None => {}
        }
    }

    let remove_whitespace = Regex::new(r" {2,}").unwrap();

    file_name = file_name
        .replace("_", " ")
        .replace("-", " ")
        .replace("+", " ");

    file_name = String::from(remove_whitespace.replace_all(&file_name, " "));

    file_name = String::from(file_name.trim());

    file_name
}

pub fn cleanse_evil_from_name(name: &str) -> String {
    String::from(
        name.replace("\\", " ")
            .replace("/", " ")
            .replace(":", " ")
            .replace("*", " ")
            .replace("?", " ")
            .replace("\"", " ")
            .replace("<", " ")
            .replace(">", " ")
            .replace("|", " ")
            .trim(),
    )
}

pub fn open_folder_in_explorer(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).output().unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).output().unwrap();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(path).output().unwrap();
    }
}

pub fn get_folder_size(path: &str) -> u64 {
    let path = PathBuf::from(path);
    std::fs::read_dir(path)
        .unwrap()
        .map(|f| f.unwrap().metadata().unwrap().len())
        .sum()
}

pub fn is_zippable_file_extension(extension: &str) -> bool {
    let lowercase = extension.to_lowercase();

    vec!["stl", "obj", "step", "gcode"]
        .iter()
        .any(|f| lowercase.as_str().eq(*f))
}

pub fn is_zipped_file_extension(extension: &str) -> bool {
    let lowercase = extension.to_lowercase();

    vec!["stl.zip", "obj.zip", "step.zip", "gcode.zip"]
        .iter()
        .any(|f| lowercase.as_str().eq(*f))
}

pub fn convert_extension_to_zip(extension: &str) -> String {
    let lowercase = extension.to_lowercase();

    String::from(match lowercase.as_str() {
        "stl" => "stl.zip",
        "obj" => "obj.zip",
        "step" => "step.zip",
        "gcode" => "gcode.zip",
        _ => &lowercase,
    })
}

pub fn convert_zip_to_extension(extension: &str) -> String {
    let lowercase = extension.to_lowercase();

    String::from(match lowercase.as_str() {
        "stl.zip" => "stl",
        "obj.zip" => "obj",
        "step.zip" => "step",
        "gcode.zip" => "gcode",
        _ => &lowercase,
    })
}

pub fn read_file_as_text(path: &PathBuf) -> Result<String, ApplicationError> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
