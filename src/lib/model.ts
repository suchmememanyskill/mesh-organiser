export type RawModelFlag = 'Printed' | 'Favorite';
export type RawModedlFlags = RawModelFlag[];

export type RawResourceFlag = 'Completed';
export type RawResourceFlags = RawResourceFlag[];

export interface ModelFlags 
{
    printed : boolean;
    favorite : boolean;
}

export interface ResourceFlags
{
    completed : boolean;
}

export function convertRawToModelFlags(raw : RawModedlFlags) : ModelFlags
{
    let flags : ModelFlags = defaultFlags();

    raw.forEach(flag => {
        switch (flag) {
            case "Printed":
                flags.printed = true;
                break;
            case "Favorite":
                flags.favorite = true;
                break;
        }
    });

    return flags;
}

export function convertModelFlagsToRaw(flags : ModelFlags) : RawModedlFlags
{
    let raw_flags : RawModedlFlags = [];

    if (flags.printed)
    {
        raw_flags.push("Printed");
    }

    if (flags.favorite)
    {
        raw_flags.push("Favorite");
    }

    return raw_flags;
}

export function convertRawToResourceFlags(raw : RawResourceFlags) : ResourceFlags
{
    let flags : ResourceFlags = {
        completed: false,
    };

    raw.forEach(flag => {
        switch (flag) {
            case "Completed":
                flags.completed = true;
                break;
        }
    });

    return flags;
}

export function convertResourceFlagsToRaw(flags : ResourceFlags) : RawResourceFlags
{
    let raw_flags : RawResourceFlags = [];

    if (flags.completed)
    {
        raw_flags.push("Completed");
    }

    return raw_flags;
}

export function defaultFlags() : ModelFlags
{
    return {
        printed : false,
        favorite : false,
    };
}

export interface RawLabelMin
{
    id : number;
    name : string;
    color : number;
}

export interface RawLabel extends RawLabelMin
{
    children : RawLabelMin[];
    effective_labels : RawLabelMin[];
    has_parent : boolean;
}

export interface RawGroup
{
    id : number;
    name : string;
    created : string;
    resource_id: number|null;
}

export interface RawModel 
{
    id : number;
    name : string;
    sha256 : string;
    filetype : FileType;
    size : number;
    link? : string;
    description? : string;
    added : string;
    group? : RawGroup;
    labels : RawLabelMin[];
    flags : RawModedlFlags;
}

export interface RawResource 
{
    id : number;
    name : string;
    flags : RawResourceFlags;
    group_ids : number[];
    created: string;
}

export interface LabelMin 
{
    id : number;
    name : string;
    color : string;
} 

export interface Label extends LabelMin
{
    children : LabelMin[];
    effectiveLabels: LabelMin[];
    hasParent: boolean;
}

export interface Resource
{
    id : number;
    name : string;
    flags : ResourceFlags;
    groups : GroupedEntry[];
    createdAt: Date;
}

export interface Model 
{
    id : number;
    name : string;
    sha256 : string;
    filetype : FileType;
    size : number;
    link? : string;
    description? : string;
    added : Date;
    labels : LabelMin[];
    flags : ModelFlags;
}

export interface Group
{
    id : number;
    name : string;
    createdAt : Date;
    resourceId: number|null;
    flags : ModelFlags;
}

export interface GroupedEntry
{
    group : Group;
    models : Model[];
    labels : LabelMin[];
    total : number;
}

export interface ModelWithGroup extends Model
{
    group? : Group;
}

export interface LabelEntry
{
    label : Label;
    entries : GroupedEntry[];
    total : number;
}

export type SizeOptionModels = "Grid_Small" | "Grid_Medium" | "Grid_Large" | "List_Small" | "List_Medium" | "List_Large";
export const SizeOptionModelsAsList = ["Grid_Small", "Grid_Medium", "Grid_Large", "List_Small", "List_Medium", "List_Large"] as SizeOptionModels[];
export type OrderOptionModels = "date-asc" | "date-desc" | "name-asc" | "name-desc" | "size-asc" | "size-desc";
export type OrderOptionGroups = "date-asc" | "date-desc" | "name-asc" | "name-desc";

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
        thumbnail_color: "#DDDDDD",
        allow_importing_step : false,
        size_option_groups : "Grid_Medium",
        size_option_models : "Grid_Medium",
        show_grouped_count_on_labels: true,
        fallback_3mf_thumbnail: true,
        prefer_3mf_thumbnail: false,
        core_parallelism: 3,
        collapse_sidebar: false,
        zoom_level: 100,
        export_metadata: false,
        show_date_on_list_view: true,
        default_enabled_delete_after_import: false,
        default_enabled_recursive_import: false,
        open_links_in_external_browser: true,
        max_size_model_3mf_preview: 15,
        max_size_model_stl_preview: 30,
        max_size_model_obj_preview: 30,
        allow_importing_gcode: true,
        only_show_single_image_in_groups: false,
        custom_slicer_path: "",
        group_split_view: "no_split",
        label_exported_model_as_printed: false,
        theme: "default",
        order_option_models: "date-desc",
        order_option_groups: "date-desc",
    }
}

export interface InitialState
{
    deep_link_url?: string;
    max_parallelism?: number;
    collapse_sidebar?: boolean;
}

export interface SlicerEntry
{
    slicer : string,
    installed : boolean,
}

export interface AddModelResult 
{
    group_id: number|null;
    model_ids: number[];
}

export interface DownloadResult 
{
    path : string;
    source_uri : string|null;
}

export enum FileType
{
    STL = "stl.zip",
    OBJ = "obj.zip",
    THREEMF = "3mf",
    STEP = "step.zip",
    GCODE = "gcode.zip",
}

export interface LabelKeyword
{
    id: number;
    name: string;
    label_id: number;
}

export enum ImportStatus {
    Idle = "Idle",
    ProcessingModels = "ProcessingModels",
    FinishedModels = "FinishedModels",
    ProcessingThumbnails = "ProcessingThumbnails",
    FinishedThumbnails = "FinishedThumbnails",
    Finished = "Finished",
    Failure = "Failure",
}

export interface ImportedModelsSet {
    group_id: number | null,
    group_name: string | null,
    model_ids: number[],
}

export interface ImportState {
    imported_models: ImportedModelsSet[],
    imported_models_count: number,
    model_count: number,
    finished_thumbnails_count: number,
    status: ImportStatus,
    origin_url: string,
    failure_reason: string | null,
    recursive: boolean,
    delete_after_import: boolean,
    current_importing_group?: string,
}