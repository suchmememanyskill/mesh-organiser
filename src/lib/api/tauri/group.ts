import { invoke } from "@tauri-apps/api/core";
import type { GroupMeta, GroupOrderBy, IGroupApi } from "../shared/services/group_api";
import type { Model } from "../shared/services/model_api";

export class GroupApi implements IGroupApi {
    async getGroups(group_ids: number[] | null, label_ids: number[] | null, order_by: GroupOrderBy, text_search: string | null, page: number, page_size: number, include_ungrouped_models: boolean): Promise<GroupMeta[]> {
        return await invoke("get_groups", {
            groupIds: group_ids,
            labelIds: label_ids,
            orderBy: order_by,
            textSearch: text_search,
            page: page,
            pageSize: page_size,
            includeUngroupedModels: include_ungrouped_models,
        })
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