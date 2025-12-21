import { dateToString } from "$lib/utils";
import type { Group, GroupMeta, GroupOrderBy, IGroupApi } from "../shared/group_api";
import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { parseRawGroup, parseRawGroupMeta, type RawGroup, type RawGroupMeta } from "../tauri/group";

export class WebGroupApi implements IGroupApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getGroups(model_ids: number[] | null, group_ids: number[] | null, label_ids: number[] | null, order_by: GroupOrderBy, text_search: string | null, page: number, page_size: number, include_ungrouped_models: boolean): Promise<Group[]> {
        let data = {
            // Hack to bypass request uri becoming too large
            model_ids_str: model_ids?.join(","),
            group_ids: group_ids,
            label_ids: label_ids,
            order_by: order_by,
            text_search: text_search,
            page: page,
            page_size: page_size,
            include_ungrouped_models: include_ungrouped_models
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