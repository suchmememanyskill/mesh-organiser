import { type RawModel, type RawGroup, type RawLabel, type Group, type Label, type GroupedEntry, type Model, type ModelWithGroup, type LabelEntry, type Configuration, configurationDefault } from "./model";
import { getLabels, getModels, getConfig } from "./tauri";

export const data = $state({
    entries : [] as ModelWithGroup[],
    grouped_entries : [] as GroupedEntry[],
    labels : [] as LabelEntry[]
});

export let c = $state({
    configuration : configurationDefault()
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

function convertGroup(raw : RawGroup) : Group
{
    return {
        id : raw.id,
        name : raw.name,
        createdAt : new Date(raw.created),
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
                    group : convertGroup(raw.group),
                    models : [],
                    total : 0,
                });
            }

            let group = groups.get(raw.group.id)!;

            let model : ModelWithGroup = {
                group: group.group,
                ...convertModel(raw),
            };

            group.models.push(model);
            group.total += 1;
        }
        else 
        {
            looseModels.push(convertModel(raw));
        }
    });

    let ret = [...groups.values()];

    if (c.configuration.show_ungrouped_models_in_groups)
    {
        looseModels.forEach(model => {
            ret.push({
                group: {
                    id: Math.random() * Number.MAX_SAFE_INTEGER * -1,
                    name: model.name,
                    createdAt: model.added,
                },
                models: [model],
                total: 1,
            });
        });
    }

    return ret;
}

function extractModels(models : RawModel[]) : ModelWithGroup[]
{
    return models.map(raw => {
        let group = undefined;

        if (raw.group)
        {
            group = convertGroup(raw.group);
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

export async function initConfiguration() : Promise<void>
{
    const config = await getConfig();
    console.log(config);
    c.configuration = config;
}