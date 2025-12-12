import { toast } from "svelte-sonner";
import { ModelOrderBy, type IModelApi, type Model } from "../shared/model_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { createShareInstance, type IShareApi, type Share } from "../shared/share_api";
import { WebShareApi } from "../web/share";

export class TauriProxyShareApi extends WebShareApi {
    private remoteModelApi : IModelApi;
    private localModelApi : IModelApi;

    constructor(requestApi : IServerRequestApi, remoteModelApi : IModelApi, localModelApi : IModelApi) {
        super(requestApi);
        this.remoteModelApi = remoteModelApi;
        this.localModelApi = localModelApi;
    }

    async getShares(): Promise<Share[]> {
        let shares = await super.getShares();
        const localModels = await this.localModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
        const remoteModels = await this.remoteModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);

        for (let share of shares) {
            const remoteGlobalIds = share.modelIds.map(id => remoteModels.find(m => m.id === id)?.uniqueGlobalId).filter(id => id !== undefined) as string[];

            if (remoteGlobalIds.length !== share.modelIds.length) {
                console.error(`Some models in share ${share.id} do not exist on the remote server`);
            }

            const localModelIds = localModels.filter(m => remoteGlobalIds.includes(m.uniqueGlobalId)).map(m => m.id);

            if (localModelIds.length !== remoteGlobalIds.length) {
                console.error(`Some models in share ${share.id} do not exist on the local server`);
            }

            share.modelIds = localModelIds;
        }

        return shares;
    }

    async addModelsToShare(share: Share, models: Model[]): Promise<void> {
        let allRemoteModels = await this.remoteModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
        let remoteModels = allRemoteModels.filter(remoteModel => models.some(localModel => localModel.uniqueGlobalId === remoteModel.uniqueGlobalId));

        if (remoteModels.length !== models.length) {
            throw new Error("Some models to add to the share do not exist on the remote server");
        }

        return super.addModelsToShare(share, remoteModels);
    }

    async setModelsOnShare(share: Share, models: Model[]): Promise<void> {
        let allRemoteModels = await this.remoteModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
        let remoteModels = allRemoteModels.filter(remoteModel => models.some(localModel => localModel.uniqueGlobalId === remoteModel.uniqueGlobalId));

        if (remoteModels.length !== models.length) {
            throw new Error("Some models to set on the share do not exist on the remote server");
        }

        return super.setModelsOnShare(share, remoteModels);
    }
}