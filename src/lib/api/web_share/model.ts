import type { ModelOrderBy, ModelFlags, Model, ModelFilter } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { Share } from "../shared/share_api";
import { convertModelFlagsToRaw, parseRawModel, type ModelApi, type RawModel } from "../tauri/model";

export class WebShareModelApi implements ModelApi {
    private requestApi : IServerRequestApi;
    private share : Share;

    constructor(requestApi : IServerRequestApi, share : Share) {
        this.requestApi = requestApi;
        this.share = share;
    }

    async getModels(filter : ModelFilter, page : number, pageSize : number): Promise<Model[]> {
        let data = {
            model_ids: filter.modelIds,
            group_ids: filter.groupIds,
            label_ids: filter.labelIds,
            order_by: filter.orderBy,
            text_search: filter.textSearch,
            file_types: filter.fileTypes,
            page: page,
            page_size: pageSize,
            model_flags: convertModelFlagsToRaw(filter.flags)
        }

        const response = await this.requestApi.request<RawModel[]>(`/shares/${this.share.id}/models`, HttpMethod.GET, data);
        return response.map(rawModel => parseRawModel(rawModel));
    }

    async editModel(model: Model): Promise<void> {
    }

    async deleteModel(model: Model): Promise<void> {
    }

    async deleteModels(models: Model[]): Promise<void> {
    }

    async getModelCount(flags: ModelFlags | null): Promise<number> {
        return this.share.modelIds.length;
    }
}