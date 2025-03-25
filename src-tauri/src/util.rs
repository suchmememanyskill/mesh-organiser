use std::path::PathBuf;
use regex::Regex;

pub fn prettify_file_name(file: &PathBuf) -> String {
    let extension = file.extension();
    let mut file_name: String = String::from(file.file_name().take().unwrap().to_str().unwrap());

    match extension {
        Some(ext) => {
            file_name = String::from(&file_name[0..file_name.len() - ext.len() - 1]);
        }
        None => {}
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


pub fn open_folder_in_explorer(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(path)
            .output()
            .unwrap();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .output()
            .unwrap();
    }
}

pub fn get_folder_size(path: &str) -> u64 {
    let path = PathBuf::from(path);
    std::fs::read_dir(path).unwrap().map(|f| f.unwrap().metadata().unwrap().len()).sum()
}