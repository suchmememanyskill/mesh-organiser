import type { GroupOrderBy, Group, GroupMeta } from "../shared/group_api";
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