#[derive(serde::Deserialize)]
pub struct EditGlobalId {
    pub new_unique_global_id: String,
}

pub mod auth_controller;
pub mod blob_controller;
pub mod group_controller;
pub mod label_controller;
pub mod model_controller;
pub mod resource_controller;
pub mod user_controller;
pub mod threemf_controller;
pub mod page_controller;
pub mod share_controller;