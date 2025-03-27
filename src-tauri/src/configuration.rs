use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::service::slicer_service::Slicer;

#[derive(Clone, Deserialize)]
pub struct StoredConfiguration {
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
    pub allow_importing_step : Option<bool>,
    pub size_option_models : Option<String>,
    pub size_option_groups : Option<String>,
}

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
    pub allow_importing_step : bool,
    pub size_option_models : String,
    pub size_option_groups : String,
}

pub fn stored_to_configuration(configuration : StoredConfiguration) -> Configuration
{
    let default = Configuration::default();

    Configuration {
        data_path: configuration.data_path,
        prusa_deep_link: configuration.prusa_deep_link,
        cura_deep_link: configuration.cura_deep_link,
        bambu_deep_link: configuration.bambu_deep_link,
        orca_deep_link: configuration.orca_deep_link,
        open_slicer_on_remote_model_import: configuration.open_slicer_on_remote_model_import,
        show_ungrouped_models_in_groups: configuration.show_ungrouped_models_in_groups,
        slicer: configuration.slicer,
        focus_after_link_import: configuration.focus_after_link_import,
        thumbnail_color: configuration.thumbnail_color,
        allow_importing_step : configuration.allow_importing_step.unwrap_or(default.allow_importing_step),
        size_option_models : configuration.size_option_models.unwrap_or(default.size_option_models),
        size_option_groups : configuration.size_option_groups.unwrap_or(default.size_option_groups),
    }
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
            allow_importing_step : false,
            size_option_models : String::from("Grid_Medium"),
            size_option_groups : String::from("Grid_Medium"),
        }
    }
}
