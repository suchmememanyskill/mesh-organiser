import { invoke } from "@tauri-apps/api/core";
import type { IModelApi, Model, ModelFlags, ModelOrderBy } from "../shared/services/model_api";

function convertModelFlagsToRaw(flags : ModelFlags|null) : string[]|null
{
    if (flags === null) {
        return null;
    }

    let raw_flags : string[] = [];

    if (flags.printed)
    {
        raw_flags.push("Printed");
    }

    if (flags.favorite)
    {
        raw_flags.push("Favorite");
    }

    if (raw_flags.length === 0) {
        return null;
    }

    return raw_flags;
}

export class ModelApi implements IModelApi {
    async getModels(model_ids: number[] | null, group_ids: number[] | null, label_ids: number[] | null, order_by: ModelOrderBy, text_search: string | null, page: number, page_size: number, flags: ModelFlags|null): Promise<Model[]> {
        return await invoke("get_models", {
            modelIds: model_ids,
            groupIds: group_ids,
            labelIds: label_ids,
            orderBy: order_by,
            textSearch: text_search,
            modelFlags: convertModelFlagsToRaw(flags),
            page: page,
            pageSize: page_size,
        })
    }

    async editModel(model: Model): Promise<void> {
        return await invoke("edit_model", { modelId: model.id, modelName: model.name, modelUrl: model.link, modelDescription: model.description, modelFlags: convertModelFlagsToRaw(model.flags) });
    }

    async deleteModel(model: Model): Promise<void> {
        return await invoke("delete_model", { modelId: model.id });
    }

    async getModelCount(flags : ModelFlags|null): Promise<number> {
        return await invoke("get_model_count", { flags: convertModelFlagsToRaw(flags) });
    }
}