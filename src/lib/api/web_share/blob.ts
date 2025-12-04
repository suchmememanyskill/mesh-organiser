import type { Blob, IBlobApi } from "../shared/blob_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { Share } from "../shared/share_api";

export class WebShareBlobApi implements IBlobApi {
    private requestApi : IServerRequestApi;
    private share : Share;

    constructor(requestApi : IServerRequestApi, share : Share) {
        this.requestApi = requestApi;
        this.share = share;
    }

    async getBlobDownloadUrl(blob: Blob): Promise<string> {
        return document.location.origin + `/api/v1/blobs/${blob.sha256}/download?share_id=${this.share.id}`;
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        return await this.requestApi.requestBinary(`/blobs/${blob.sha256}/download?share_id=${this.share.id}`, HttpMethod.GET);
    }

    async getBlobThumbnailUrl(blob: Blob): Promise<string> {
        return document.location.origin + "/api/v1/blobs/" + blob.sha256 + "/thumb";
    }
}