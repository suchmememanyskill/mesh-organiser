import { IDiskUsageInfoApi, type DiskUsageInfo } from "../shared/disk_usage_info_api";
import { mockModels } from "./mock_data";

export class DemoDiskUsageInfoApi implements IDiskUsageInfoApi {
    async getDiskUsageInfo(): Promise<DiskUsageInfo> {
        // Calculate total size from all mock models
        let totalSize = 0;
        for (const model of mockModels.values()) {
            totalSize += model.blob.size;
        }

        return {
            size_uncompressed: totalSize,
            size_compressed: Math.floor(totalSize * 0.5) // 50% of uncompressed
        };
    }
}
