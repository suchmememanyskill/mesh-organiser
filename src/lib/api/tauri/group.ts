import { invoke } from "@tauri-apps/api/core";
import { Group, GroupMeta, type GroupOrderBy, type IGroupApi } from "../shared/services/group_api";
import { Model } from "../shared/services/model_api";
import type { RawLabelMeta } from "./label";
import type { RawModel } from "./model";
import type { RawResourceMeta } from "./resource";
import { Blob } from "../shared/services/blob_api";
import { LabelMeta } from "../shared/services/label_api";
import { ResourceMeta } from "../shared/services/resource_api";

export interface RawGroupMeta {
    id: number;
    name: string;
    created: string;
    resource_id: number|null;
}

export interface RawGroup {
    meta: RawGroupMeta;
    models: RawModel[];
    labels: RawLabelMeta[];
    resource: RawResourceMeta|null;
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

        return groups.map(group => new Group(
            new GroupMeta(
                group.meta.id,
                group.meta.name,
                group.meta.created,
            ),
            group.models.map(model => new Model(
                model.id,
                model.name,
                new Blob(
                    model.blob.id,
                    model.blob.sha256,
                    model.blob.filetype,
                    model.blob.size,
                    model.blob.added
                ),
                model.link,
                model.description,
                model.added,
                model.group ? new GroupMeta(
                    model.group.id,
                    model.group.name,
                    model.group.created,
                ) : null,
                model.labels.map(label => new LabelMeta(
                    label.id,
                    label.name,
                    label.color
                )),
                model.flags
            )),
            group.labels.map(label => new LabelMeta(
                label.id,
                label.name,
                label.color
            )),
            group.resource ? new ResourceMeta(
                group.resource.id,
                group.resource.name,
                group.resource.flags,
                group.resource.created,
            ) : null,
        ));
    }

    async addGroup(name: string): Promise<number> {
        return await invoke("add_group", { group_name: name });
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