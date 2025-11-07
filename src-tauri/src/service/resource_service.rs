use db::model::{Resource, ResourceMeta, User};
use crate::error::ApplicationError;
use crate::util::open_folder_in_explorer;

use super::app_state::AppState;

pub async fn open_resource_folder(
    resource: &ResourceMeta,
    user: &User,
    app_state: &AppState,
) -> Result<(), ApplicationError> {
    let path = app_state.get_resources_dir();
    let resource_path = path.join(format!("{}_{}", resource.id, user.id));

    if !path.exists() {
        let old_resource_path = path.join(resource.id.to_string());
        if old_resource_path.exists() {
            std::fs::rename(&old_resource_path, &resource_path)?;
        } else {
            std::fs::create_dir_all(&path)?;
        }
    }

    open_folder_in_explorer(path.to_str().unwrap());

    Ok(())
}

pub async fn delete_resource_folder(
    resource: &ResourceMeta,
    user: &User,
    app_state: &AppState,
) -> Result<(), ApplicationError> {
    let mut path = app_state.get_resources_dir();
    path.push(format!("{}_{}", resource.id, user.id));

    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }

    Ok(())
}
