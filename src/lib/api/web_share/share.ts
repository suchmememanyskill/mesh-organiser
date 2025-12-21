import type { Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { IShareApi, Share } from "../shared/share_api";
import { parseRawShare, type RawShare } from "../web/share";

export class LimitedWebShareApi implements IShareApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }
    
    async getShares(): Promise<Share[]> {
        return [];
    }
    
    async getShare(shareId: string): Promise<Share> {
        let rawShare = await this.requestApi.request<RawShare>(`/shares/${shareId}`, HttpMethod.GET);
        return parseRawShare(rawShare);
    }
    
    async getShareLink(share: Share): Promise<string> {
        throw new Error("Method not implemented.");
    }
    
    async createShare(shareName: string): Promise<Share> {
        throw new Error("Method not implemented.");
    }
    
    async addModelsToShare(share: Share, models: Model[]): Promise<void> {
    }
    
    async setModelsOnShare(share: Share, models: Model[]): Promise<void> {
    }
    
    async editShare(share: Share): Promise<void> {
    }
    
    async deleteShare(share: Share): Promise<void> {
    }
}