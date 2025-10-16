import { type RawModel, type RawGroup, type RawLabel, type Group, type Label, type GroupedEntry, type Model, type ModelWithGroup, type LabelEntry, type Configuration, configurationDefault, convertRawToModelFlags, type LabelMin, type RawLabelMin, type Resource, convertRawToResourceFlags, type RawResource } from "./model";
import { getLabels, getModels, getConfig, setConfig, getResources } from "./tauri";
import { debounce } from "./utils";
import { emit } from "@tauri-apps/api/event";

export const data = $state({
    entries : [] as ModelWithGroup[],
    grouped_entries : [] as GroupedEntry[],
    labels : [] as LabelEntry[],
    resources : [] as Resource[],
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
        labels : raw.labels.map(label => convertLabelMin(label)),
        flags : convertRawToModelFlags(raw.flags),
    };
}

function convertGroup(raw : RawGroup) : Group
{
    return {
        id : raw.id,
        name : raw.name,
        createdAt : new Date(raw.created),
        resourceId : raw.resource_id,
        flags : {
            printed : true,
            favorite : true,
        },
    };
}

function convertLabel(raw : RawLabel) : Label
{
    return {
        ...convertLabelMin(raw),
        children : raw.children.map(child => convertLabelMin(child)),
        effectiveLabels : raw.effective_labels.map(label => convertLabelMin(label)),
        hasParent : raw.has_parent,
    }
}

function convertLabelMin(raw : RawLabelMin) : LabelMin
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
                    labels : [],
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

            if (group.group.flags.printed && !model.flags.printed)
            {
                group.group.flags.printed = false;
            }

            if (group.group.flags.favorite && !model.flags.favorite)
            {
                group.group.flags.favorite = false;
            }
        }
        else 
        {
            looseModels.push(convertModel(raw));
        }
    });

    groups.forEach(group => group.labels = group.models
        .flatMap(model => model.labels)
        .filter((label, index, self) => self.findIndex(l => l.id === label.id) === index));

    let ret = [...groups.values()];

    if (c.configuration.show_ungrouped_models_in_groups)
    {
        looseModels.forEach(model => {
            ret.push({
                group: {
                    id: model.id * -1,
                    name: model.name,
                    createdAt: model.added,
                    flags: model.flags,
                    resourceId: null,
                },
                models: [model],
                labels: model.labels,
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

function extractResources(rawResources : RawResource[], groupedEntries : GroupedEntry[]) : Resource[]
{
    let groupMap : Map<number, GroupedEntry> = new Map();

    groupedEntries.forEach(group => {
        groupMap.set(group.group.id, group);
    });

    return rawResources.map(rawResource => {
        let groups = rawResource.group_ids.map(groupId => groupMap.get(groupId)) as GroupedEntry[];

        let resource: Resource = {
            id: rawResource.id,
            name: rawResource.name,
            flags: convertRawToResourceFlags(rawResource.flags),
            groups: groups,
            createdAt: new Date(rawResource.created),
        };

        groups.forEach(group => group.group.resourceId = resource.id);
        return resource;
    });
}

export async function updateState() : Promise<void>
{
    let start = performance.now();
    let rawModels = await getModels();
    let rawLabels = await getLabels();
    let rawResources = await getResources();
    let afterFetch = performance.now();
    console.log(rawLabels);
    
    let modelGroups = extractGroups(rawModels);
    let models = extractModels(rawModels);
    let resources = extractResources(rawResources, modelGroups);

    // TODO: Make this more efficient
    let labels : LabelEntry[] = rawLabels.map(raw_label => {
        let label = convertLabel(raw_label);
        
        let filtered_models = models.filter(model => model.labels.some(l => label.effectiveLabels.some(el => el.id === l.id)));

        let grouped_filtered_models : any = {};
        let singles = [];
        let filter_groups : GroupedEntry[] = [];

        for (const model of filtered_models)
        {
            if (model.group)
            {
                if (!grouped_filtered_models[model.group.id])
                {
                    grouped_filtered_models[model.group.id] = []
                }

                grouped_filtered_models[model.group.id].push(model);
            }
            else
            {
                singles.push(model);
            }
        }

        for (const [key, value] of Object.entries(grouped_filtered_models))
        {
            const group_id = parseInt(key);
            const group = modelGroups.find(g => g.group.id === group_id);

            if (group && group.total === (value as Model[]).length)
            {
                filter_groups.push(group);
            }
            else 
            {
                (value as Model[]).forEach(model => singles.push(model));
            }
        }

        let singles_as_groups : GroupedEntry[] = singles.map(model => { 
            return {
                group : {
                    id: model.id * -1,
                    name: model.name,
                    createdAt: model.added,
                    flags: model.flags,
                    resourceId: null,
                },
                labels : model.labels,
                models : [model],
                total : 1,
            }
        });


        return {
            label : label,
            entries : [...filter_groups, ...singles_as_groups],
            total : filtered_models.length,
        };
    });

    data.entries = models;
    data.grouped_entries = modelGroups;
    data.labels = labels;
    data.resources = resources;

    console.log("Update took", performance.now() - start, "ms,", afterFetch - start, "ms for fetching data.");
    await emit("state-change", {});
}

export async function initConfiguration() : Promise<void>
{
    const config = await getConfig();
    console.log(config);
    c.configuration = config;
}

export const on_save_configuration = debounce(
    async (edited_configuration: Configuration) => {
        console.log("Setting config", edited_configuration);
        await setConfig(edited_configuration);
    },
    500,
);