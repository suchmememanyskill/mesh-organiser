use std::num::NonZeroUsize;
use std::thread;

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
    pub allow_importing_step: Option<bool>,
    pub size_option_models: Option<String>,
    pub size_option_groups: Option<String>,
    pub show_grouped_count_on_labels : Option<bool>,
    pub fallback_3mf_thumbnail: Option<bool>,
    pub prefer_3mf_thumbnail: Option<bool>,
    pub core_parallelism: Option<usize>,
    pub collapse_sidebar : Option<bool>,
    pub zoom_level: Option<u32>,
    pub export_metadata: Option<bool>,
    pub show_date_on_list_view: Option<bool>,
    pub default_enabled_recursive_import: Option<bool>,
    pub default_enabled_delete_after_import: Option<bool>,
    pub open_links_in_external_browser: Option<bool>,
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
    pub allow_importing_step: bool,
    pub size_option_models: String,
    pub size_option_groups: String,
    pub show_grouped_count_on_labels: bool,
    pub fallback_3mf_thumbnail: bool,
    pub prefer_3mf_thumbnail: bool,
    pub core_parallelism: usize,
    pub collapse_sidebar: bool,
    pub zoom_level: u32,
    pub export_metadata: bool,
    pub show_date_on_list_view: bool,
    pub default_enabled_recursive_import: bool,
    pub default_enabled_delete_after_import: bool,
    pub open_links_in_external_browser: bool,
}

pub fn stored_to_configuration(configuration: StoredConfiguration) -> Configuration {
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
        allow_importing_step: configuration
            .allow_importing_step
            .unwrap_or(default.allow_importing_step),
        size_option_models: configuration
            .size_option_models
            .unwrap_or(default.size_option_models),
        size_option_groups: configuration
            .size_option_groups
            .unwrap_or(default.size_option_groups),
        show_grouped_count_on_labels: configuration
            .show_grouped_count_on_labels
            .unwrap_or(default.show_grouped_count_on_labels),
        fallback_3mf_thumbnail: configuration
            .fallback_3mf_thumbnail
            .unwrap_or(default.fallback_3mf_thumbnail),
        prefer_3mf_thumbnail: configuration
            .prefer_3mf_thumbnail
            .unwrap_or(default.prefer_3mf_thumbnail),
        collapse_sidebar: configuration
            .collapse_sidebar
            .unwrap_or(default.collapse_sidebar),
        zoom_level: configuration
            .zoom_level
            .unwrap_or(default.zoom_level),
        export_metadata: configuration
            .export_metadata
            .unwrap_or(default.export_metadata),
        show_date_on_list_view: configuration
            .show_date_on_list_view
            .unwrap_or(default.show_date_on_list_view),
        core_parallelism: configuration
            .core_parallelism
            .unwrap_or(default.core_parallelism),
        default_enabled_recursive_import: configuration
            .default_enabled_recursive_import
            .unwrap_or(default.default_enabled_recursive_import),
        default_enabled_delete_after_import: configuration
            .default_enabled_delete_after_import
            .unwrap_or(default.default_enabled_delete_after_import),
        open_links_in_external_browser: configuration
            .open_links_in_external_browser
            .unwrap_or(default.open_links_in_external_browser),
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
            allow_importing_step: false,
            size_option_models: String::from("Grid_Medium"),
            size_option_groups: String::from("Grid_Medium"),
            show_grouped_count_on_labels: true,
            fallback_3mf_thumbnail: true,
            prefer_3mf_thumbnail: false,
            core_parallelism: thread::available_parallelism().unwrap_or(NonZeroUsize::new(6).unwrap()).get() / 2,
            collapse_sidebar: false,
            zoom_level: 100,
            export_metadata: false,
            show_date_on_list_view: true,
            default_enabled_recursive_import: false,
            default_enabled_delete_after_import: false,
            open_links_in_external_browser: true,
        }
    }
}
