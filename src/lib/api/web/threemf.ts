import type { GroupMeta } from "../shared/group_api";
import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { IThreemfApi, ThreemfMetadata } from "../shared/threemf_api";
import { parseRawGroupMeta, type RawGroupMeta } from "../tauri/group";

export class WebThreemfApi implements IThreemfApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getThreemfMetadata(modelId: Model): Promise<ThreemfMetadata | null> {

        try {
            return await this.requestApi.request<ThreemfMetadata>(`/models/${modelId.id}/3mf_metadata`, HttpMethod.GET);
        }
        catch {
            return null;
        }
    }

    async extractThreemfModels(modelId: Model): Promise<GroupMeta> {
        let groupMeta = await this.requestApi.request<RawGroupMeta>(`/models/${modelId.id}/3mf_extract`, HttpMethod.POST);

        return parseRawGroupMeta(groupMeta);
    }
}