import { invoke } from "@tauri-apps/api/core";

interface RawLabel
{
    id : number;
    name : string;
    color : number;
}

interface RawGroup
{
    id : number;
    name : string;
}

interface RawModel 
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

// Either a single model or a group of models
export interface GroupedEntry
{
    group? : Group;
    models : Model[];
    total : number;
}

export interface LabelEntry
{
    label : Label;
    entries : GroupedEntry[];
    total : number;
}

export const models = $state({
    entries : [] as GroupedEntry[],
    labels : [] as LabelEntry[],
});


function convertModel(raw : RawModel) : Model
{
    return {
        id : raw.id,
        name : raw.name,
        sha256 : raw.sha256,
        filetype : raw.filetype,
        size : raw.size,
        link : raw.link,
        description : raw.description,
        added : new Date(raw.added),
        labels : raw.labels.map(label => convertLabel(label)),
    };
}

function convertGroup(raw : RawGroup, raw_model : RawModel) : Group
{
    return {
        id : raw.id,
        name : raw.name,
        createdAt : new Date(raw_model.added),
    };
}

function convertLabel(raw : RawLabel) : Label
{
    return {
        id : raw.id,
        name : raw.name,
        color : `#${raw.color.toString(16).padStart(6, "0")}`,
    };
}

function extractGroups(models : RawModel[]) : GroupedEntry[]
{
    let looseModels : Model[] = [];
    let groups : Map<number, GroupedEntry> = new Map();

    models.forEach(raw => {
        if (raw.group)
        {
            if (!groups.has(raw.group.id))
            {
                groups.set(raw.group.id, {
                    group : convertGroup(raw.group, raw),
                    models : [],
                    total : 0,
                });
            }

            let group = groups.get(raw.group.id)!;

            group.models.push(convertModel(raw));
            group.total += 1;
        }
        else 
        {
            looseModels.push(convertModel(raw));
        }
    });

    return [...groups.values(), ...looseModels.map(loose => ({ models: [ loose ], total: 1 }))];
}

function filterGroupOnLabel(group : GroupedEntry, label : Label) : GroupedEntry
{
    let models = group.models.filter(model => model.labels.some(l => l.id === label.id));

    return {
        group : group.group,
        models : models,
        total : models.length,
    };
}

export async function updateState() : Promise<void>
{
    let raw_models : RawModel[] = await invoke("get_models");
    let raw_labels : RawLabel[] = await invoke("get_labels");

    console.log(raw_models);

    let model_groups = extractGroups(raw_models);

    let labels : LabelEntry[] = raw_labels.map(raw_label => {
        let label = convertLabel(raw_label);
        let groups = model_groups.map(group => filterGroupOnLabel(group, label)).filter(group => group.total > 0);

        return {
            label : label,
            entries : groups,
            total : groups.reduce((acc, entry) => acc + entry.total, 0),
        };
    });

    models.labels = labels;
    models.entries = model_groups;
}