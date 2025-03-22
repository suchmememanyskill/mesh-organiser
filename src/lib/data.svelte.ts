import type { RawModel, RawGroup, RawLabel, Group, Label, GroupedEntry, Model, ModelWithGroup, LabelEntry } from "./model";
import { getLabels, getModels } from "./tauri";

export const data = $state({
    entries : [] as ModelWithGroup[],
    grouped_entries : [] as GroupedEntry[],
    labels : [] as LabelEntry[]
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

    return [...groups.values()];
}

function extractModels(models : RawModel[]) : ModelWithGroup[]
{
    return models.map(raw => {
        let group = undefined;

        if (raw.group)
        {
            group = convertGroup(raw.group, raw);
        }

        return {
            group : group,
            ...convertModel(raw),
        };
    });
}

export async function updateState() : Promise<void>
{
    let raw_models = await getModels();
    let raw_labels = await getLabels();

    let model_groups = extractGroups(raw_models);
    let models = extractModels(raw_models);

    let labels : LabelEntry[] = raw_labels.map(raw_label => {
        let label = convertLabel(raw_label);
        let filtered_models = models.filter(model => model.labels.some(l => l.id === label.id));

        return {
            label : label,
            entries : filtered_models,
            total : filtered_models.length,
        };
    });

    console.log(model_groups);
    console.log(models);
    console.log(labels);

    data.entries = models;
    data.grouped_entries = model_groups;
    data.labels = labels;
}