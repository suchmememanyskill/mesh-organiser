export interface DiskUsageInfo {
    size_uncompressed: number;
    size_compressed: number;
}

export const IDiskUsageInfoApi = Symbol('IDiskUsageInfoApi');

export interface IDiskUsageInfoApi {
    getDiskUsageInfo() : Promise<DiskUsageInfo>;
}