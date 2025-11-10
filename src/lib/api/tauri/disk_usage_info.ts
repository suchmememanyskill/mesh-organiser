import { invoke } from "@tauri-apps/api/core";
import type { DiskUsageInfo, IDiskUsageInfoApi } from "../shared/disk_usage_info_api";

export class DiskUsageInfoApi implements IDiskUsageInfoApi {
    async getDiskUsageInfo(): Promise<DiskUsageInfo> {
        return invoke("get_model_disk_space_usage");
    }
}