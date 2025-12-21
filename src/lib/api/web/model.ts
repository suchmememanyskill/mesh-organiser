import { dateToString } from "$lib/utils";
import type { IModelApi, Model, ModelFlags, ModelOrderBy } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { convertModelFlagsToRaw, parseRawModel, type RawModel } from "../tauri/model";

export class WebModelApi implements IModelApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
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

        const response = await this.requestApi.request<RawModel[]>("/models", HttpMethod.GET, data);
        return response.map(rawModel => parseRawModel(rawModel));
    }
    
    async editModel(model: Model, editTimestamp?: boolean, editGlobalId?: boolean): Promise<void> {
        let data : any = {
            model_name: model.name,
            model_url: model.link,
            model_description: model.description,
            model_flags: convertModelFlagsToRaw(model.flags)
        };

        if (editTimestamp) {
            data.model_timestamp = dateToString(model.lastModified);
        }

        if (editGlobalId) {
            data.model_global_id = model.uniqueGlobalId;
        }

        await this.requestApi.request<void>(`/models/${model.id}`, HttpMethod.PUT, data);
    }

    async deleteModel(model: Model): Promise<void> {
        await this.requestApi.request<void>(`/models/${model.id}`, HttpMethod.DELETE);
    }

    async deleteModels(models: Model[]): Promise<void> {
        let data = {
            model_ids: models.map(model => model.id)
        }

        await this.requestApi.request<void>(`/models`, HttpMethod.DELETE, data);
    }

    async getModelCount(flags: ModelFlags | null): Promise<number> {
        let data = {
            model_flags: convertModelFlagsToRaw(flags)
        };

        return (await this.requestApi.request<any>("/models/count", HttpMethod.GET, data)).count;
    }
}