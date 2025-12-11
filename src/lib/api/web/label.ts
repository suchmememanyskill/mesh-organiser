import { dateToString } from "$lib/utils";
import { stringColorToNumber, type ILabelApi, type Label, type LabelMeta } from "../shared/label_api";
import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { parseRawLabel, parseRawLabelMeta, type RawLabel, type RawLabelKeyword, type RawLabelMeta } from "../tauri/label";

export class WebLabelApi implements ILabelApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getLabels(includeUngroupedModels: boolean): Promise<Label[]> {
        let data = {
            include_ungrouped_models: includeUngroupedModels
        }
        let labels = await this.requestApi.request<RawLabel[]>(`/labels`, HttpMethod.GET, data);
        return labels.map(rawLabel => parseRawLabel(rawLabel));
    }

    async addLabel(name: string, color: string): Promise<LabelMeta> {
        let data = {
            label_name: name,
            label_color: stringColorToNumber(color)
        }

        return parseRawLabelMeta(await this.requestApi.request<RawLabelMeta>("/labels", HttpMethod.POST, data));
    }

    async editLabel(label: LabelMeta, editTimestamp?: boolean, editGlobalId?: boolean): Promise<void> {
        let data : any = {
            label_name: label.name,
            label_color: stringColorToNumber(label.color)
        }

        if (editTimestamp) {
            data.label_timestamp = dateToString(label.lastModified);
        }

        if (editGlobalId) {
            data.label_global_id = label.uniqueGlobalId;
        }

        await this.requestApi.request<void>(`/labels/${label.id}`, HttpMethod.PUT, data);
    }

    async deleteLabel(label: LabelMeta): Promise<void> {
        await this.requestApi.request<void>(`/labels/${label.id}`, HttpMethod.DELETE);
    }
    
    async setLabelsOnModel(Labels: LabelMeta[], model: Model): Promise<void> {
        let data = {
            label_ids: Labels.map(label => label.id)
        }

        await this.requestApi.request<void>(`/models/${model.id}/labels`, HttpMethod.PUT, data);
    }

    async addLabelToModels(label: LabelMeta, models: Model[]): Promise<void> {
        let data = {
            model_ids: models.map(model => model.id)
        }

        await this.requestApi.request<void>(`/labels/${label.id}/models`, HttpMethod.POST, data);
    }

    async removeLabelFromModels(label: LabelMeta, models: Model[]): Promise<void> {
        let data = {
            model_ids: models.map(model => model.id)
        }

        await this.requestApi.request<void>(`/labels/${label.id}/models`, HttpMethod.DELETE, data);
    }

    async setKeywordsOnLabel(label: LabelMeta, keywords: string[]): Promise<void> {
        let data = {
            keywords: keywords
        }

        await this.requestApi.request<void>(`/labels/${label.id}/keywords`, HttpMethod.PUT, data);
    }

    async getKeywordsForLabel(label: LabelMeta): Promise<string[]> {
        return (await this.requestApi.request<RawLabelKeyword[]>(`/labels/${label.id}/keywords`, HttpMethod.GET)).map(kw => kw.name);
    }

    async setChildrenOnLabel(label: LabelMeta, children: LabelMeta[]): Promise<void> {
        let data = {
            child_label_ids: children.map(child => child.id)
        }

        await this.requestApi.request<void>(`/labels/${label.id}/childs`, HttpMethod.PUT, data);
    }

    async removeChildrenFromLabel(label: LabelMeta, children: LabelMeta[]): Promise<void> {
        let data = {
            child_label_ids: children.map(child => child.id)
        }

        await this.requestApi.request<void>(`/labels/${label.id}/childs`, HttpMethod.DELETE, data);
    }
}