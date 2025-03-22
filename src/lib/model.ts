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
    entries : ModelWithGroup[];
    total : number;
}

export enum SupportedSlicers {
    PrusaSlicer = "PrusaSlicer",
    Cura = "Cura",
    BambuStudio = "BambuStudio",
    OrcaSlicer = "OrcaSlicer",
}

export interface Configuration {
    dataPath: string;
    modelPath: string;
    prusaDeepLink: boolean;
    curaDeepLink: boolean;
    bambuDeepLink: boolean;
    orcaDeepLink: boolean;
    slicer: SupportedSlicers;
    createPopupOnModelImport: boolean;
}

export interface InitialState
{
    deep_link_url?: string;
}