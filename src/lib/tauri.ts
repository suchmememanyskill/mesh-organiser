import { invoke } from "@tauri-apps/api/core";
import { type RawModel, type RawLabel, type RawGroup, type Model, type Group, type Label, type InitialState, type Configuration, type SlicerEntry, type AddModelResult, type DownloadResult, type RawModedlFlags, convertModelFlagsToRaw, defaultFlags, type LabelMin, type RawResource, convertResourceFlagsToRaw, type Resource, type LabelKeyword, type ImportState } from "./model";

export async function getModels() : Promise<RawModel[]>
{
    let raw_models : RawModel[] = await invoke("get_models");

    return raw_models;
}

export async function getLabels() : Promise<RawLabel[]>
{
    let raw_labels : RawLabel[] = await invoke("get_labels");

    return raw_labels;
}

export async function getResources() : Promise<RawResource[]>
{
    let raw_resources : RawResource[] = await invoke("get_resources");

    return raw_resources;
}

export async function editModel(model : Model) : Promise<void>
{
    await invoke("edit_model", {  
        modelId: model.id,
        modelName: model.name,
        modelDescription: model.description,
        modelUrl: model.link,
        modelFlags: convertModelFlagsToRaw(model.flags),
    });
}

export async function deleteModel(model : Model) : Promise<void>
{
    await invoke("delete_model", { modelId: model.id });
}

export async function createLabel(name : string, color : string) : Promise<LabelMin>
{
    let colorHex = color.replace("#", "");
    let colorNumber = parseInt(colorHex, 16);

    let labelId : number = await invoke("add_label", { labelName: name, labelColor: colorNumber });
    return {
        id: labelId,
        name: name,
        color: color,
    }
}

export async function ungroup(group : Group) : Promise<void>
{
    await invoke("ungroup", { groupId: group.id });
}

export async function editGroup(group : Group) : Promise<void>
{
    await invoke("edit_group", { groupId: group.id, groupName: group.name, groupResourceId: group.resourceId });
}

export async function setLabelsOnModel(labels : LabelMin[], model : Model) : Promise<void>
{
    let labelIds = labels.map(label => label.id);

    await invoke("set_labels_on_model", { modelId: model.id, labelIds: labelIds });
}

export async function openInSlicer(models : Model[]) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("open_in_slicer", { modelIds: modelIds });
}

export async function getInitialState() : Promise<InitialState>
{
    return await invoke("get_initial_state");
}

export async function downloadFile(url : string) : Promise<DownloadResult>
{
    return await invoke("download_file", { url: url });
}

export async function openInFolder(models : Model[]) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("open_in_folder", { modelIds: modelIds });
}

export async function setLabelOnModels(models : Model[], label : LabelMin) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("set_label_on_models", { modelIds: modelIds, labelId: label.id });
}

export async function removeLabelFromModels(models : Model[], label : LabelMin) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("remove_label_from_models", { modelIds: modelIds, labelId: label.id });
}

export async function addEmptyGroup(group_name : string) : Promise<Group>
{
    let group_id : number = await invoke("add_group", { groupName: group_name });

    return {
        id: group_id,
        name: group_name,
        createdAt: new Date(),
        flags: defaultFlags(),
        resourceId: null,
    }
}

export async function addModelsToGroup(models : Model[], group : Group) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("add_models_to_group", { modelIds: modelIds, groupId: group.id });
}

export async function removeModelsFromGroup(models : Model[], group : Group) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("remove_models_from_group", { modelIds: modelIds, groupId: group.id });
}

export async function removeDeadGroups() : Promise<void>
{
    await invoke("remove_dead_groups");
}

export async function editLabel(label : LabelMin) : Promise<void>
{
    let colorHex = label.color.replace("#", "");
    let colorNumber = parseInt(colorHex, 16);

    await invoke("edit_label", { labelId: label.id, labelName: label.name, labelColor: colorNumber });
}

export async function deleteLabel(label : LabelMin) : Promise<void>
{
    await invoke("delete_label", { labelId: label.id });
}

export async function getConfig() : Promise<Configuration>
{
    return await invoke("get_configuration");
}

export async function setConfig(config : Configuration) : Promise<void>
{
    if (!config.slicer)
    {
        config.slicer = null;
    }

    await invoke("set_configuration", { configuration: config});
}

export async function getAvailableSlicers() : Promise<SlicerEntry[]>
{
    return await invoke("get_slicers");
}

export async function updateImages(overwrite : boolean) : Promise<void>
{
    await invoke("update_images", { overwrite: overwrite });
}

export async function importModel(path : string, recursive : boolean, delete_imported : boolean, origin_url : string|null, open_in_slicer: boolean) : Promise<ImportState>
{
    return await invoke("add_model", {
        path: path,
        recursive : recursive,
        deleteImported : delete_imported,
        originUrl : origin_url,
        openInSlicer: open_in_slicer,
    });
}

export async function computeModelFolderSize() : Promise<number>
{
    return await invoke("compute_model_folder_size");
}

export async function newWindow(url : string) : Promise<void>
{
    await invoke("new_window_with_url", { url: url });
}

export async function getModelAsBase64(model : Model) : Promise<string>
{
    return await invoke("get_model_as_base64", { modelId: model.id });
}

export async function getModelBytes(model : Model) : Promise<Uint8Array>
{
    return await invoke("get_model_bytes", { modelId: model.id });
}

export async function addChildsToLabel(parent : LabelMin, childs : LabelMin[]) : Promise<void>
{
    let childIds = childs.map(child => child.id);

    await invoke("add_childs_from_label", { parentLabelId: parent.id, childLabelIds: childIds });
}

export async function removeChildsFromLabel(parent : LabelMin, childs : LabelMin[]) : Promise<void>
{
    let childIds = childs.map(child => child.id);

    await invoke("remove_childs_from_label", { parentLabelId: parent.id, childLabelIds: childIds });
}

export async function setChildsOnLabel(parent : LabelMin, childs : LabelMin[]) : Promise<void>
{
    let childIds = childs.map(child => child.id);

    await invoke("set_childs_on_label", { parentLabelId: parent.id, childLabelIds: childIds });
}

export async function editResource(resource: Resource): Promise<void>
{
    await invoke("edit_resource", {
        resourceId: resource.id,
        resourceName: resource.name,
        resourceFlags: convertResourceFlagsToRaw(resource.flags),
    });
}

export async function deleteResource(resource: Resource): Promise<void>
{
    await invoke("remove_resource", { resourceId: resource.id });
}

export async function addResource(name: string): Promise<Resource>
{
    let resourceId: number = await invoke("add_resource", { resourceName: name });
    
    return {
        id: resourceId,
        name: name,
        flags: { completed: false },
        groups: [],
        createdAt: new Date(),
    };
}

export async function openResourceFolder(resource: Resource): Promise<void>
{
    await invoke("open_resource_folder", { resourceId: resource.id });
}

export async function setKeywordsOnLabel(label: LabelMin, keywords: string[]): Promise<void>
{
    await invoke("set_keywords_on_label", { labelId: label.id, keywords: keywords });
}

export async function getKeywordsForLabel(label: LabelMin): Promise<LabelKeyword[]>
{
    return await invoke("get_keywords_for_label", { labelId: label.id });
}