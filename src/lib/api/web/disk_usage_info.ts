import type { DiskUsageInfo, IDiskUsageInfoApi } from "../shared/disk_usage_info_api";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";

export class WebDiskUsageInfoApi implements IDiskUsageInfoApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getDiskUsageInfo(): Promise<DiskUsageInfo> {
        return await this.requestApi.request<DiskUsageInfo>("/models/disk_usage", HttpMethod.GET);
    }
}