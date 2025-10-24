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
    pub show_grouped_count_on_labels: Option<bool>,
    pub fallback_3mf_thumbnail: Option<bool>,
    pub prefer_3mf_thumbnail: Option<bool>,
    pub core_parallelism: Option<usize>,
    pub collapse_sidebar: Option<bool>,
    pub zoom_level: Option<u32>,
    pub export_metadata: Option<bool>,
    pub show_date_on_list_view: Option<bool>,
    pub default_enabled_recursive_import: Option<bool>,
    pub default_enabled_delete_after_import: Option<bool>,
    pub open_links_in_external_browser: Option<bool>,
    pub max_size_model_3mf_preview: Option<u32>,
    pub max_size_model_stl_preview: Option<u32>,
    pub max_size_model_obj_preview: Option<u32>,
    pub allow_importing_gcode: Option<bool>,
    pub only_show_single_image_in_groups: Option<bool>,
    pub custom_slicer_path: Option<String>,
    pub elegoo_deep_link: Option<bool>,
    pub group_split_view: Option<String>,
    pub label_exported_model_as_printed: Option<bool>,
    pub theme: Option<String>,
    pub order_option_models: Option<String>,
    pub order_option_groups: Option<String>,
    pub ignore_update: Option<String>,
    pub show_multiselect_checkboxes: Option<bool>,
    pub use_worker_for_model_parsing: Option<bool>,
    pub prefer_gcode_thumbnail: Option<bool>,
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
    pub max_size_model_3mf_preview: u32,
    pub max_size_model_stl_preview: u32,
    pub max_size_model_obj_preview: u32,
    pub allow_importing_gcode: bool,
    pub only_show_single_image_in_groups: bool,
    pub custom_slicer_path: String,
    pub elegoo_deep_link: bool,
    pub group_split_view: String,
    pub label_exported_model_as_printed: bool,
    pub theme: String,
    pub order_option_models: String,
    pub order_option_groups: String,
    pub ignore_update: String,
    pub show_multiselect_checkboxes: bool,
    pub use_worker_for_model_parsing: bool,
    pub prefer_gcode_thumbnail: bool,
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
        zoom_level: configuration.zoom_level.unwrap_or(default.zoom_level),
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
        max_size_model_3mf_preview: configuration
            .max_size_model_3mf_preview
            .unwrap_or(default.max_size_model_3mf_preview),
        max_size_model_stl_preview: configuration
            .max_size_model_stl_preview
            .unwrap_or(default.max_size_model_stl_preview),
        max_size_model_obj_preview: configuration
            .max_size_model_obj_preview
            .unwrap_or(default.max_size_model_obj_preview),
        allow_importing_gcode: configuration
            .allow_importing_gcode
            .unwrap_or(default.allow_importing_gcode),
        only_show_single_image_in_groups: configuration
            .only_show_single_image_in_groups
            .unwrap_or(default.only_show_single_image_in_groups),
        custom_slicer_path: configuration
            .custom_slicer_path
            .unwrap_or(default.custom_slicer_path),
        elegoo_deep_link: configuration
            .elegoo_deep_link
            .unwrap_or(default.elegoo_deep_link),
        group_split_view: configuration
            .group_split_view
            .unwrap_or(default.group_split_view),
        label_exported_model_as_printed: configuration
            .label_exported_model_as_printed
            .unwrap_or(default.label_exported_model_as_printed),
        theme: configuration
            .theme
            .unwrap_or(default.theme),
        order_option_models: configuration
            .order_option_models
            .unwrap_or(default.order_option_models),
        order_option_groups: configuration
            .order_option_groups
            .unwrap_or(default.order_option_groups),
        ignore_update: configuration
            .ignore_update
            .unwrap_or(default.ignore_update),
        show_multiselect_checkboxes: configuration
            .show_multiselect_checkboxes
            .unwrap_or(default.show_multiselect_checkboxes),
        use_worker_for_model_parsing: configuration
            .use_worker_for_model_parsing
            .unwrap_or(default.use_worker_for_model_parsing),
        prefer_gcode_thumbnail: configuration
            .prefer_gcode_thumbnail
            .unwrap_or(default.prefer_gcode_thumbnail),
    }
}

impl Default for Configuration {
    fn default() -> Self {
        let installed_slicer = Slicer::iter().find(|f| f.is_installed());
        let mut parallelism = thread::available_parallelism()
            .unwrap_or(NonZeroUsize::new(3).unwrap())
            .get()
            / 2;

        if parallelism <= 0 {
            parallelism = 1;
        }

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
            core_parallelism: parallelism,
            collapse_sidebar: false,
            zoom_level: 100,
            export_metadata: false,
            show_date_on_list_view: true,
            default_enabled_recursive_import: false,
            default_enabled_delete_after_import: false,
            open_links_in_external_browser: true,
            max_size_model_3mf_preview: 15,
            max_size_model_stl_preview: 30,
            max_size_model_obj_preview: 30,
            allow_importing_gcode: true,
            only_show_single_image_in_groups: false,
            custom_slicer_path: String::new(),
            elegoo_deep_link: false,
            group_split_view: String::from("no_split"),
            label_exported_model_as_printed: false,
            theme: String::from("default"),
            order_option_models: String::from("date-desc"),
            order_option_groups: String::from("date-desc"),
            ignore_update: String::from(""),
            show_multiselect_checkboxes: true,
            use_worker_for_model_parsing: true,
            prefer_gcode_thumbnail: true,
        }
    }
}
