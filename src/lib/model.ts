export interface RawLabel
{
    id : number;
    name : string;
    color : number;
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
    labels : RawLabel[];
}

export interface Label 
{
    id : number;
    name : string;
    color : string;
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
    labels : Label[];
}

export interface Group
{
    id : number;
    name : string;
    createdAt : Date;
}

export interface GroupedEntry
{
    group : Group;
    models : Model[];
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
    }
}

export interface InitialState
{
    deep_link_url?: string;
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