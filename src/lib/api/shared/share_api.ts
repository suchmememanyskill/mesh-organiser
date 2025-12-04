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