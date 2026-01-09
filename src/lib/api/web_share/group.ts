import type { GroupOrderBy, Group, GroupMeta, GroupFilter } from "../shared/group_api";
import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { Share } from "../shared/share_api";
import { parseRawGroup, type GroupApi, type RawGroup } from "../tauri/group";

export class WebShareGroupApi implements GroupApi {
    private requestApi : IServerRequestApi;
    private share : Share;

    constructor(requestApi : IServerRequestApi, share : Share) {
        this.requestApi = requestApi;
        this.share = share;
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

        const response = await this.requestApi.request<RawGroup[]>(`/shares/${this.share.id}/groups`, HttpMethod.GET, data);
        return response.map(rawGroup => parseRawGroup(rawGroup));
    }
    
    async addGroup(name: string): Promise<GroupMeta> {
        throw new Error("Method not implemented.");
    }
    
    async editGroup(group: GroupMeta): Promise<void> {
    }
    
    async deleteGroup(group: GroupMeta): Promise<void> {
    }
    
    async addModelsToGroup(group: GroupMeta, models: Model[]): Promise<void> {
    }
    
    async removeModelsFromGroup(models: Model[]): Promise<void> {
    }
    
    async getGroupCount(include_ungrouped_models: boolean): Promise<number> {
        return this.share.modelIds.length;
    }
}