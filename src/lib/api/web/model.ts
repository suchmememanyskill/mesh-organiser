import { dateToString } from "$lib/utils";
import type { IModelApi, Model, ModelFilter, ModelFlags, ModelOrderBy } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { convertModelFlagsToRaw, parseRawModel, type RawModel } from "../tauri/model";

export class WebModelApi implements IModelApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
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