export type RawFlags = 'Printed'[];

export interface Flags 
{
    printed : boolean;
}

export function convertRawToFlags(raw : RawFlags) : Flags
{
    let flags : Flags = defaultFlags();

    raw.forEach(flag => {
        switch (flag) {
            case "Printed":
                flags.printed = true;
                break;
        }
    });

    return flags;
}

export function convertFlagsToRaw(flags : Flags) : RawFlags
{
    let raw_flags : RawFlags = [];

    if (flags.printed)
    {
        raw_flags.push("Printed");
    }

    return raw_flags;
}

export function defaultFlags() : Flags
{
    return {
        printed : false,
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
}

export interface RawModel 
{
    id : number;
    name : string;
    sha256 : string;
    filetype : string;
    size : number;
    link? : string;
    description? : string;
    added : string;
    group? : RawGroup;
    labels : RawLabelMin[];
    flags : RawFlags;
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

export interface Model 
{
    id : number;
    name : string;
    sha256 : string;
    filetype : string;
    size : number;
    link? : string;
    description? : string;
    added : Date;
    labels : LabelMin[];
    flags : Flags;
}

export interface Group
{
    id : number;
    name : string;
    createdAt : Date;
    flags : Flags;
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

export interface Configuration {
    data_path: string;
    prusa_deep_link: boolean;
    cura_deep_link: boolean;
    bambu_deep_link: boolean;
    orca_deep_link: boolean;
    open_slicer_on_remote_model_import: boolean;
    show_ungrouped_models_in_groups: boolean;
    slicer: string|null;
    focus_after_link_import: boolean;
    thumbnail_color : string;
    allow_importing_step : boolean;
    size_option_models : "Grid_Small" | "Grid_Medium" | "Grid_Large" | "List_Small" | "List_Medium" | "List_Large";
    size_option_groups : "Grid_Small" | "Grid_Medium" | "Grid_Large" | "List_Small" | "List_Medium" | "List_Large";
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
    show_models_from_child_label_in_parent_labels : boolean;
}

export function configurationDefault() : Configuration
{
    return {
        data_path: "",
        prusa_deep_link: false,
        cura_deep_link: false,
        bambu_deep_link: false,
        orca_deep_link: false,
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
        show_models_from_child_label_in_parent_labels: true,
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
}