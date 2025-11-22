import type { GroupMeta } from "./group_api";
import type { Model } from "./model_api";

export interface ThreemfMetadata {
    nozzle_diameter: number | null;
    layer_height: number | null;
    material_type: string | null;
    supports_enabled: boolean | null;
}

export const IThreemfApi = Symbol('IThreemfApi');

export interface IThreemfApi {
    getThreemfMetadata(modelId: Model) : Promise<ThreemfMetadata|null>;
    extractThreemfModels(modelId: Model) : Promise<GroupMeta>;
}