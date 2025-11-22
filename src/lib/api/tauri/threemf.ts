import { invoke } from "@tauri-apps/api/core";
import type { IThreemfApi, ThreemfMetadata } from "../shared/threemf_api";
import type { Model } from "../shared/model_api";

export class ThreemfApi implements IThreemfApi {
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