import type { Blob, IBlobApi } from "../shared/blob_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { User } from "../shared/user_api";

export class WebBlobApi implements IBlobApi {
    private requestApi : IServerRequestApi;
    private user : User;

    constructor(requestApi : IServerRequestApi, currentUser: User) {
        this.requestApi = requestApi;
        this.user = currentUser;
    }

    async getBlobDownloadUrl(blob: Blob): Promise<string> {
        return document.location.origin + `/api/v1/blobs/${blob.sha256}/download?user_hash=${this.user.syncUrl}&user_id=${this.user.id}`;
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        return await this.requestApi.requestBinary("/blobs/" + blob.sha256 + "/bytes", HttpMethod.GET);
    }

    async getBlobThumbnailUrl(blob: Blob): Promise<string> {
        return document.location.origin + "/api/v1/blobs/" + blob.sha256 + "/thumb";
    }
}