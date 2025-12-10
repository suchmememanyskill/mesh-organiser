import { invoke } from "@tauri-apps/api/core";
import { createGroupInstance, createGroupMetaInstance, type Group, type GroupMeta, type GroupOrderBy, type IGroupApi } from "../shared/group_api";
import { type Model } from "../shared/model_api";
import { parseRawLabelMeta, type RawLabelMeta } from "./label";
import { parseRawModel, type RawModel } from "./model";
import { parseRawResourceMeta, type RawResourceMeta } from "./resource";
import { dateToString } from "$lib/utils";

export interface RawGroupMeta {
    id: number;
    name: string;
    created: string;
    last_modified: string;
    resource_id: number|null;
    unique_global_id: string;
}

export function parseRawGroupMeta(raw: RawGroupMeta): GroupMeta {
    return createGroupMetaInstance(
        raw.id,
        raw.name,
        raw.created,
        raw.last_modified,
        raw.unique_global_id
    );
}

export interface RawGroup {
    meta: RawGroupMeta;
    models: RawModel[];
    labels: RawLabelMeta[];
    resource: RawResourceMeta|null;
    flags: string[];
}

export function parseRawGroup(raw: RawGroup): Group {
    return createGroupInstance(
        parseRawGroupMeta(raw.meta),
        raw.models.map(model => parseRawModel(model)),
        raw.labels.map(label => parseRawLabelMeta(label)),
        raw.resource ? parseRawResourceMeta(raw.resource) : null,
        raw.flags
    );
}
        

export class GroupApi implements IGroupApi {
    async getGroups(model_ids: number[]|null, group_ids: number[] | null, label_ids: number[] | null, order_by: GroupOrderBy, text_search: string | null, page: number, page_size: number, include_ungrouped_models: boolean): Promise<Group[]> {
        let groups = await invoke<RawGroup[]>("get_groups", {
            modelIds: model_ids,
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
        let group = await invoke<RawGroupMeta>("add_group", { groupName: name });
        return parseRawGroupMeta(group);
    }

    async editGroup(group: GroupMeta, editTimestamp?: boolean, editGlobalId?: boolean): Promise<void> {
        let data : any = { 
            groupId: group.id, 
            groupName: group.name 
        }
        
        if (editTimestamp) {
            data.groupTimestamp = dateToString(group.lastModified);
        }

        if (editGlobalId) {
            data.groupGlobalId = group.uniqueGlobalId;
        }

        return await invoke("edit_group", data);
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