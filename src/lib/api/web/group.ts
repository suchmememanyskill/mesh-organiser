import { dateToString } from "$lib/utils";
import type { Group, GroupFilter, GroupMeta, GroupOrderBy, IGroupApi } from "../shared/group_api";
import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { parseRawGroup, parseRawGroupMeta, type RawGroup, type RawGroupMeta } from "../tauri/group";

export class WebGroupApi implements IGroupApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getGroups(filter : GroupFilter, page: number, page_size: number): Promise<Group[]> {
        let data = {
            // Hack to bypass request uri becoming too large
            model_ids_str: filter.modelIds?.join(","),
            group_ids: filter.groupIds,
            label_ids: filter.labelIds,
            order_by: filter.orderBy,
            text_search: filter.textSearch,
            file_types: filter.fileTypes,
            page: page,
            page_size: page_size,
            include_ungrouped_models: filter.includeUngroupedModels
        }

        const response = await this.requestApi.request<RawGroup[]>("/groups", HttpMethod.GET, data);
        return response.map(rawGroup => parseRawGroup(rawGroup));
    }

    async addGroup(name: string): Promise<GroupMeta> {
        let data = {
            group_name: name
        }

        const response = await this.requestApi.request<RawGroupMeta>("/groups", HttpMethod.POST, data);
        return parseRawGroupMeta(response);
    }
    
    async editGroup(group: GroupMeta, editTimestamp?: boolean, editGlobalId?: boolean): Promise<void> {
        let data : any = {
            group_name: group.name
        }

        if (editTimestamp) {
            data.group_timestamp = dateToString(group.lastModified);
        }

        if (editGlobalId) {
            data.group_global_id = group.uniqueGlobalId;
        }

        await this.requestApi.request<void>(`/groups/${group.id}`, HttpMethod.PUT, data);
    }

    async deleteGroup(group: GroupMeta): Promise<void> {
        await this.requestApi.request<void>(`/groups/${group.id}`, HttpMethod.DELETE);
    }

    async addModelsToGroup(group: GroupMeta, models: Model[]): Promise<void> {
        let data = {
            model_ids: models.map(model => model.id)
        }

        await this.requestApi.request<void>(`/groups/${group.id}/models`, HttpMethod.POST, data);
    }

    async removeModelsFromGroup(models: Model[]): Promise<void> {
        let data = {
            model_ids: models.map(model => model.id)
        }

        await this.requestApi.request<void>(`/groups/detach_models`, HttpMethod.DELETE, data);
    }

    async getGroupCount(include_ungrouped_models: boolean): Promise<number> {
        let data = {
            include_ungrouped_models: include_ungrouped_models
        }

        return (await this.requestApi.request<any>("/groups/count", HttpMethod.GET, data)).count;
    }
}