use serde::Serialize;

use crate::service::slicer_service::Slicer;

#[derive(Clone, Serialize)]
pub struct Configuration {
    pub data_path: String,
    pub model_path: String,
    pub prusa_deep_link: bool,
    pub cura_deep_link: bool,
    pub bambu_deep_link: bool,
    pub orca_deep_link: bool,
    pub open_slicer_on_remote_model_import: bool,
    pub slicer: Slicer,
    pub create_popup_on_model_import: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            data_path: String::from(""),
            model_path: String::from(""),
            prusa_deep_link: true,
            cura_deep_link: true,
            bambu_deep_link: true,
            orca_deep_link: true,
            open_slicer_on_remote_model_import: false,
            slicer: Slicer::OrcaSlicer,
            create_popup_on_model_import: false,
        }
    }
}
