import { toast } from "svelte-sonner";
import { IGroupApi, type GroupMeta } from "./group_api";
import type { Model } from "./model_api";
import { updateSidebarState } from "$lib/sidebar_data.svelte";
import { goto } from "$app/navigation";

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

export async function extractThreemfModels(model : Model, threemfApi : IThreemfApi|null, groupApi: IGroupApi|null) : Promise<void>
{
    if (!threemfApi || !groupApi) {
        return;
    }

    let promise = threemfApi.extractThreemfModels(model);

    toast.promise(
        promise,
        {
            loading: `Extracting models from '${model.name}'...`,
            success: (newGroup) => {
                return `Imported '${newGroup.name}''`;
            },
        }
    );

    let newGroup = await promise;
    await groupApi.addModelsToGroup(newGroup, [model]);
    model.group = newGroup;
    await updateSidebarState();

    await goto("/group/" + newGroup.id);
}