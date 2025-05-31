use crate::db::resource::Resource;
use crate::error::ApplicationError;
use crate::util::open_folder_in_explorer;

use super::app_state::AppState;

pub async fn open_resource_folder(
    resource : &Resource, 
    app_state: &AppState
) -> Result<(), ApplicationError> {
    let mut path = app_state.get_resources_dir();
    path.push(resource.id.to_string());
    
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    open_folder_in_explorer(path.to_str().unwrap());
    
    Ok(())
}

pub async fn delete_resource_folder(
    resource: &Resource, 
    app_state: &AppState
) -> Result<(), ApplicationError> {
    let mut path = app_state.get_resources_dir();
    path.push(resource.id.to_string());

    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }

    Ok(())
}