import type { Blob, IBlobApi } from "../shared/blob_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";

export class WebBlobApi implements IBlobApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        return await this.requestApi.requestBinary("/blobs/" + blob.sha256 + "/bytes", HttpMethod.GET);
    }

    async getBlobThumbnailUrl(blob: Blob): Promise<string> {
        return document.location.origin + "/api/v1/blobs/" + blob.sha256 + "/thumb";
    }
}