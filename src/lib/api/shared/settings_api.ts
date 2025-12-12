import { GroupOrderBy } from "./group_api";
import { ModelOrderBy } from "./model_api";

export type SizeOptionModels = "Grid_Small" | "Grid_Medium" | "Grid_Large" | "List_Small" | "List_Medium" | "List_Large";
export const SizeOptionModelsAsList = ["Grid_Small", "Grid_Medium", "Grid_Large", "List_Small", "List_Medium", "List_Large"] as SizeOptionModels[];
export type OrderOptionModels = "date-asc" | "date-desc" | "name-asc" | "name-desc" | "size-asc" | "size-desc" | "modified-asc" | "modified-desc";
export type OrderOptionGroups = "date-asc" | "date-desc" | "name-asc" | "name-desc" | "modified-asc" | "modified-desc";

export interface Configuration {
    data_path: string;
    prusa_deep_link: boolean;
    cura_deep_link: boolean;
    bambu_deep_link: boolean;
    orca_deep_link: boolean;
    elegoo_deep_link: boolean;
    open_slicer_on_remote_model_import: boolean;
    show_ungrouped_models_in_groups: boolean;
    slicer: string|null;
    focus_after_link_import: boolean;
    thumbnail_color : string;
    allow_importing_step : boolean;
    size_option_models : SizeOptionModels;
    size_option_groups : SizeOptionModels;
    show_grouped_count_on_labels: boolean;
    fallback_3mf_thumbnail: boolean;
    prefer_3mf_thumbnail: boolean;
    core_parallelism: number;
    collapse_sidebar: boolean;
    zoom_level: number;
    export_metadata: boolean;
    show_date_on_list_view: boolean;
    default_enabled_delete_after_import: boolean;
    default_enabled_recursive_import: boolean;
    open_links_in_external_browser: boolean;
    max_size_model_3mf_preview: number; // in MB
    max_size_model_stl_preview: number; // in MB
    max_size_model_obj_preview: number; // in MB
    allow_importing_gcode: boolean;
    only_show_single_image_in_groups: boolean;
    custom_slicer_path : string;
    group_split_view: "no_split" | "split-left-right" | "split-top-bottom";
    label_exported_model_as_printed : boolean;
    theme : string;
    order_option_models : OrderOptionModels;
    order_option_groups : OrderOptionGroups;
    ignore_update : string;
    show_multiselect_checkboxes : boolean;
    use_worker_for_model_parsing : boolean;
    prefer_gcode_thumbnail : boolean;
    custom_css : string;
    default_enabled_import_as_path : boolean;
    thumbnail_rotation : [number, number, number];
    watch_downloads_folder: boolean;
}

export function convertOrderOptionModelsToEnum(orderOption : OrderOptionModels) : ModelOrderBy {
    switch (orderOption) {
        case "date-asc":
            return ModelOrderBy.AddedAsc;
        case "date-desc":
            return ModelOrderBy.AddedDesc;
        case "name-asc":
            return ModelOrderBy.NameAsc;
        case "name-desc":
            return ModelOrderBy.NameDesc;
        case "size-asc":
            return ModelOrderBy.SizeAsc;
        case "size-desc":
            return ModelOrderBy.SizeDesc;
        case "modified-asc":
            return ModelOrderBy.ModifiedAsc;
        case "modified-desc":
            return ModelOrderBy.ModifiedDesc;
        default:
            return ModelOrderBy.AddedDesc;
    }
}

export function convertOrderOptionGroupsToEnum(orderOption : OrderOptionGroups) : GroupOrderBy {
    switch (orderOption) {
        case "date-asc":
            return GroupOrderBy.CreatedAsc;
        case "date-desc":
            return GroupOrderBy.CreatedDesc;
        case "name-asc":
            return GroupOrderBy.NameAsc;
        case "name-desc":
            return GroupOrderBy.NameDesc;
        case "modified-asc":
            return GroupOrderBy.ModifiedAsc;
        case "modified-desc":
            return GroupOrderBy.ModifiedDesc;
        default:
            return GroupOrderBy.CreatedDesc;
    }
}

export function configurationDefault() : Configuration
{
    return {
        data_path: "",
        prusa_deep_link: false,
        cura_deep_link: false,
        bambu_deep_link: false,
        orca_deep_link: false,
        elegoo_deep_link: false,
        open_slicer_on_remote_model_import: false,
        show_ungrouped_models_in_groups: true,
        slicer: null,
        focus_after_link_import: true,
        thumbnail_color: "#EEEEEE",
        allow_importing_step : true,
        size_option_groups : "Grid_Medium",
        size_option_models : "Grid_Medium",
        show_grouped_count_on_labels: true,
        fallback_3mf_thumbnail: true,
        prefer_3mf_thumbnail: true,
        core_parallelism: 3,
        collapse_sidebar: false,
        zoom_level: 100,
        export_metadata: false,
        show_date_on_list_view: false,
        default_enabled_delete_after_import: false,
        default_enabled_recursive_import: false,
        open_links_in_external_browser: true,
        max_size_model_3mf_preview: 15,
        max_size_model_stl_preview: 30,
        max_size_model_obj_preview: 30,
        allow_importing_gcode: true,
        only_show_single_image_in_groups: true,
        custom_slicer_path: "",
        group_split_view: "split-left-right",
        label_exported_model_as_printed: false,
        theme: "default",
        order_option_models: "modified-desc",
        order_option_groups: "modified-desc",
        ignore_update: "",
        show_multiselect_checkboxes: true,
        use_worker_for_model_parsing: true,
        prefer_gcode_thumbnail: true,
        custom_css : "",
        default_enabled_import_as_path : false,
        thumbnail_rotation : [35, 30, 0],
        watch_downloads_folder: false,
    }
}

export enum SettingSection {
    ThumbnailGeneration,
    ModelPreview,
    ImportExport,
    DeepLink,
    CustomSlicer,
    Behaviour,
    WindowZoom,
    UserInterface,
    Users,
    ThumbnailGenerationColorSection,
    BehaviourSectionAllPlatforms,
    CurrentUser,
}

export const ISettingsApi = Symbol('ISettingsApi');

export interface ISettingsApi {
    getConfiguration() : Promise<Configuration>;
    saveConfiguration(config: Configuration) : Promise<void>;
    availableSections() : SettingSection[];
}