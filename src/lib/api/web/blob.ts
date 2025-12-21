import type { Blob, IBlobApi } from "../shared/blob_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { User } from "../shared/user_api";

export class WebBlobApi implements IBlobApi {
    private requestApi : IServerRequestApi;
    private user : User;
    private hostUrl : string;

    constructor(requestApi : IServerRequestApi, currentUser: User, hostUrl?: string) {
        this.requestApi = requestApi;
        this.user = currentUser;
        this.hostUrl = hostUrl || document.location.origin;
    }

    async getBlobDownloadUrl(blob: Blob): Promise<string> {
        return this.hostUrl + `/api/v1/blobs/${blob.sha256}/download?user_hash=${this.user.syncUrl}&user_id=${this.user.id}`;
    }

    async getBlobsDownloadUrl(blobs: Blob[]): Promise<string> {
        let data = blobs.map(b => b.sha256);
        let zipDir = await this.requestApi.request<string>("/blobs/download", HttpMethod.POST, data);
        return this.hostUrl + `/api/v1/blobs/download/${zipDir}`;
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        return await this.requestApi.requestBinary("/blobs/" + blob.sha256 + "/bytes", HttpMethod.GET);
    }

    async getBlobThumbnailUrl(blob: Blob): Promise<string> {
        return this.hostUrl + "/api/v1/blobs/" + blob.sha256 + "/thumb";
    }
}