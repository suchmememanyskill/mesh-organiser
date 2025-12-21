import { toast } from "svelte-sonner";
import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { createShareInstance, type IShareApi, type Share } from "../shared/share_api";

export interface RawShare {
    id: string;
    share_name: string;
    created_at: string;
    user_name: string;
    model_ids: number[];
}

export function parseRawShare(raw: RawShare): Share {
    return {
        ...createShareInstance(
            raw.id,
            raw.created_at,
            raw.share_name,
            raw.user_name,
            raw.model_ids
        )
    };
}

export class WebShareApi implements IShareApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getShares(): Promise<Share[]> {
        let shares = await this.requestApi.request<RawShare[]>(`/shares`, HttpMethod.GET);
        return shares.map(rawShare => parseRawShare(rawShare));
    }

    async getShare(shareId: string): Promise<Share> {
        // Lazy implementation, should be fixed later

        let shares = await this.getShares();
        for (let share of shares) {
            if (share.id === shareId) {
                return share;
            }
        }

        return Promise.reject("Share not found");
    }

    async createShare(shareName: string): Promise<Share> {
        let data = {
            share_name: shareName
        };
        
        let rawShare = await this.requestApi.request<RawShare>(`/shares`, HttpMethod.POST, data);
        return parseRawShare(rawShare);
    }

    async addModelsToShare(share: Share, models: Model[]): Promise<void> {
        let currentModelIds = new Set<number>(share.modelIds);

        for (let model of models) {
            currentModelIds.add(model.id);
        }

        let data = {
            model_ids: Array.from(currentModelIds)
        }

        await this.requestApi.request<void>(`/shares/${share.id}/models`, HttpMethod.PUT, data);
    }

    async setModelsOnShare(share: Share, models: Model[]): Promise<void> {
        let data = {
            model_ids: models.map(m => m.id)
        };

        await this.requestApi.request<void>(`/shares/${share.id}/models`, HttpMethod.PUT, data);
    }

    async editShare(share: Share): Promise<void> {
        let data = {
            share_name: share.shareName
        };

        await this.requestApi.request<void>(`/shares/${share.id}`, HttpMethod.PUT, data);
    }

    async deleteShare(share: Share): Promise<void> {
        await this.requestApi.request<void>(`/shares/${share.id}`, HttpMethod.DELETE);
    }

    async getShareLink(share: Share): Promise<string> {
        return `${this.requestApi.baseUrl}/share/${share.id}`
    }
}