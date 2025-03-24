use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::service::slicer_service::Slicer;

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub data_path: String,
    pub prusa_deep_link: bool,
    pub cura_deep_link: bool,
    pub bambu_deep_link: bool,
    pub orca_deep_link: bool,
    pub open_slicer_on_remote_model_import: bool,
    pub show_ungrouped_models_in_groups: bool,
    pub slicer: Option<Slicer>,
    pub focus_after_link_import: bool,
    pub thumbnail_color: String,
}

impl Default for Configuration {
    fn default() -> Self {
        let installed_slicer = Slicer::iter().find(|f| f.is_installed());

        Configuration {
            data_path: String::from(""),
            prusa_deep_link: false,
            cura_deep_link: false,
            bambu_deep_link: false,
            orca_deep_link: false,
            open_slicer_on_remote_model_import: false,
            show_ungrouped_models_in_groups: true,
            slicer: installed_slicer,
            focus_after_link_import: true,
            thumbnail_color: String::from("#EEEEEE"),
        }
    }
}
