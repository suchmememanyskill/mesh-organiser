use std::path::PathBuf;

pub fn prettify_file_name(file : &PathBuf) -> String
{
    let extension = file.extension();
    let mut file_name : String = String::from(file.file_name().take().unwrap().to_str().unwrap());

    match extension
    {
        Some(ext) => 
        {
            file_name = String::from(&file_name[0..file_name.len() - ext.len()]);
        }
        None => 
        {

        }
    }

    file_name = file_name.replace("_", " ").replace("-", " ");
    
    file_name
}