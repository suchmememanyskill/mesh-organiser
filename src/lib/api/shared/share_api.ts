import { updateSidebarState } from "$lib/sidebar_data.svelte";
import { nameCollectionOfModels } from "$lib/utils";
import { toast } from "svelte-sonner";
import { getContainer } from "../dependency_injection";
import type { Model } from "./model_api";

export const IShareApi = Symbol("IShareApi");

export interface Share {
    id: string;
    createdAt: Date;
    shareName: string;
    userName: string;
    modelIds: number[];
}

export function createShareInstance(id: string, createdAt: string, shareName: string, userName: string, modelIds: number|number[]): Share {
    if (!Array.isArray(modelIds)) {
        modelIds = [modelIds];
    }

    return {
        id,
        createdAt: new Date(createdAt),
        shareName,
        userName,
        modelIds: modelIds
    };
}

export interface IShareApi {
    getShares() : Promise<Share[]>;
    getShare(shareId : string) : Promise<Share>;
    getShareLink(share : Share) : Promise<string>;
    createShare(shareName: string) : Promise<Share>;
    addModelsToShare(share : Share, models: Model[]) : Promise<void>;
    setModelsOnShare(share : Share, models: Model[]) : Promise<void>;
    editShare(share : Share) : Promise<void>;
    deleteShare(share : Share) : Promise<void>;
}

export async function createShare(models : Model[], shareApi : IShareApi|null) : Promise<void>
{
    if (!shareApi) {
        return;
    }

    let existingShares = await shareApi.getShares();
    let modelIds = models.map(m => m.id);
    let sameShare = existingShares.find(share => share.modelIds.length === modelIds.length && share.modelIds.every(id => modelIds.includes(id)));

    let share;

    if (sameShare) {
        share = sameShare;
    } else {
        share = await shareApi.createShare(nameCollectionOfModels(models));
        await shareApi.setModelsOnShare(share, models);
        await updateSidebarState();
    }
    
    let link = await shareApi.getShareLink(share);
    await navigator.clipboard.writeText(link);

    toast.success("Share created successfully. Link has been copied to your clipboard.");
}