import { invoke } from "@tauri-apps/api/core";
import type { RawModel, RawLabel, RawGroup, Model, Group, Label, InitialState } from "./model";

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

export async function editModel(model : Model) : Promise<void>
{
    await invoke("edit_model", {  
        modelId: model.id,
        modelName: model.name,
        modelDescription: model.description,
        modelUrl: model.link
    });
}

export async function deleteModel(model : Model) : Promise<void>
{
    await invoke("delete_model", { modelId: model.id });
}

export async function createLabel(name : string, color : string) : Promise<void>
{
    let colorHex = color.replace("#", "");
    let colorNumber = parseInt(colorHex, 16);

    await invoke("add_label", { labelName: name, labelColor: colorNumber });
}

export async function ungroup(group : Group) : Promise<void>
{
    await invoke("ungroup", { groupId: group.id });
}

export async function editGroup(group : Group) : Promise<void>
{
    await invoke("edit_group", { groupId: group.id, groupName: group.name });
}

export async function setLabelsOnModel(labels : Label[], model : Model) : Promise<void>
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

export async function downloadFile(url : string) : Promise<string>
{
    return await invoke("download_file", { url: url });
}

export async function openInFolder(models : Model[]) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("open_in_folder", { modelIds: modelIds });
}

export async function setLabelOnModels(models : Model[], label : Label) : Promise<void>
{
    let modelIds = models.map(model => model.id);

    await invoke("set_label_on_models", { modelIds: modelIds, labelId: label.id });
}

export async function removeLabelFromModels(models : Model[], label : Label) : Promise<void>
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
        createdAt: new Date()
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

export async function editLabel(label : Label) : Promise<void>
{
    let colorHex = label.color.replace("#", "");
    let colorNumber = parseInt(colorHex, 16);

    await invoke("edit_label", { labelId: label.id, labelName: label.name, labelColor: colorNumber });
}

export async function deleteLabel(label : Label) : Promise<void>
{
    await invoke("delete_label", { labelId: label.id });
}