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
        .replace("-", " ");

    file_name = String::from(remove_whitespace.replace_all(&file_name, " "));

    file_name
}
