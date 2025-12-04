import type { ModelOrderBy, ModelFlags, Model } from "../shared/model_api";
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

    async getModels(model_ids: number[] | null, group_ids: number[] | null, label_ids: number[] | null, order_by: ModelOrderBy, text_search: string | null, page: number, page_size: number, flags: ModelFlags | null): Promise<Model[]> {
        let data = {
            model_ids: model_ids,
            group_ids: group_ids,
            label_ids: label_ids,
            order_by: order_by,
            text_search: text_search,
            page: page,
            page_size: page_size,
            model_flags: convertModelFlagsToRaw(flags)
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