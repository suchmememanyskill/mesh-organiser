import { invoke } from "@tauri-apps/api/core";
import type { Model } from "../shared/model_api";
import type { ISlicerApi, SlicerEntry } from "../shared/slicer_api";

export class OnlineSlicerApi implements ISlicerApi {
    private baseUrl: string;
    private userId: number;
    private userHash: string;

    constructor(baseUrl: string, userId: number, userHash: string) {
        this.baseUrl = baseUrl;
        this.userId = userId;
        this.userHash = userHash;
    }

    async openInSlicer(models: Model[]): Promise<void> {
        await invoke("download_files_and_open_in_slicer", { sha256s: models.map(m => m.blob.sha256), baseUrl: this.baseUrl, userId: this.userId, userHash: this.userHash });
    }

    async availableSlicers(): Promise<SlicerEntry[]> {
        return await invoke("get_slicers", {});
    }
}