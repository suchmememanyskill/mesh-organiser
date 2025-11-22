import { invoke } from "@tauri-apps/api/core";
import type { IThreemfApi, ThreemfMetadata } from "../shared/threemf_api";
import type { Model } from "../shared/model_api";
import type { GroupMeta } from "../shared/group_api";
import { parseRawGroupMeta, type RawGroupMeta } from "./group";

export class ThreemfApi implements IThreemfApi {
    async extractThreemfModels(modelId: Model): Promise<GroupMeta> {
        let groupMeta = await invoke<RawGroupMeta>('extract_threemf_models', { modelId: modelId.id });
        return parseRawGroupMeta(groupMeta);
    }

    async getThreemfMetadata(model: Model): Promise<ThreemfMetadata|null> {
        try {
            return await invoke<ThreemfMetadata>('get_theemf_metadata', { modelId: model.id });
        }
        catch (e) {
            console.error(e);
            return null;
        }
    }
}