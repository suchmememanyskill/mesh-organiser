import { invoke } from "@tauri-apps/api/core";
import { Group, GroupMeta, type GroupOrderBy, type IGroupApi } from "../shared/services/group_api";
import { Model } from "../shared/services/model_api";
import { parseRawLabelMeta, type RawLabelMeta } from "./label";
import { parseRawModel, type RawModel } from "./model";
import { parseRawResourceMeta, type RawResourceMeta } from "./resource";
import { Blob } from "../shared/services/blob_api";
import { LabelMeta } from "../shared/services/label_api";
import { ResourceMeta } from "../shared/services/resource_api";
import { parse } from "svelte/compiler";

export interface RawGroupMeta {
    id: number;
    name: string;
    created: string;
    resource_id: number|null;
}

export function parseRawGroupMeta(raw: RawGroupMeta): GroupMeta {
    return new GroupMeta(
        raw.id,
        raw.name,
        raw.created,
    );
}

export interface RawGroup {
    meta: RawGroupMeta;
    models: RawModel[];
    labels: RawLabelMeta[];
    resource: RawResourceMeta|null;
}

export function parseRawGroup(raw: RawGroup): Group {
    return new Group(
        parseRawGroupMeta(raw.meta),
        raw.models.map(model => parseRawModel(model)),
        raw.labels.map(label => parseRawLabelMeta(label)),
        raw.resource ? parseRawResourceMeta(raw.resource) : null,
    );
}
        

export class GroupApi implements IGroupApi {
    async getGroups(group_ids: number[] | null, label_ids: number[] | null, order_by: GroupOrderBy, text_search: string | null, page: number, page_size: number, include_ungrouped_models: boolean): Promise<Group[]> {
        let groups = await invoke<RawGroup[]>("get_groups", {
            groupIds: group_ids,
            labelIds: label_ids,
            orderBy: order_by,
            textSearch: text_search,
            page: page,
            pageSize: page_size,
            includeUngroupedModels: include_ungrouped_models,
        });

        return groups.map(group => parseRawGroup(group));
    }

    async addGroup(name: string): Promise<GroupMeta> {
        let group = await invoke<RawGroupMeta>("add_group", { group_name: name });
        return parseRawGroupMeta(group);
    }

    async editGroup(group: GroupMeta): Promise<void> {
        return await invoke("edit_group", { groupId: group.id, groupName: group.name });
    }

    async deleteGroup(group: GroupMeta): Promise<void> {
        return await invoke("ungroup", { groupId: group.id });
    }

    async addModelsToGroup(group: GroupMeta, models: Model[]): Promise<void> {
        return await invoke("add_models_to_group", { groupId: group.id, modelIds: models.map(model => model.id) });
    }

    async removeModelsFromGroup(models: Model[]): Promise<void> {
        return await invoke("remove_models_from_group", { modelIds: models.map(model => model.id) });
    }

    async getGroupCount(include_ungrouped_models: boolean): Promise<number> {
        return await invoke("get_group_count", { includeUngroupedModels: include_ungrouped_models });
    }
}