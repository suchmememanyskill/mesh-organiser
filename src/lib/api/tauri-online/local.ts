import { invoke } from "@tauri-apps/api/core";
import type { ILocalApi } from "../shared/local_api";
import type { Model } from "../shared/model_api";
import { open } from "@tauri-apps/plugin-dialog";
import { join } from "@tauri-apps/api/path";
import { openPath } from "@tauri-apps/plugin-opener";

// TODO: Split this off into chunks. We don't need the data picker, appdatadir or max parallelism here!
export class OnlineLocalApi implements ILocalApi {
    appDataDir : string;
    maxParallelism : number;
    private baseUrl: string;
    private userId: number;
    private userHash: string;

    constructor(appDataDir: string, maxParallelism: number, baseUrl: string, userId: number, userHash: string) {
        this.appDataDir = appDataDir;
        this.maxParallelism = maxParallelism;
        this.baseUrl = baseUrl;
        this.userId = userId;
        this.userHash = userHash;
    }

    async openInFolder(models: Model[], asZip: boolean): Promise<void> {
        await invoke("download_files_and_open_in_folder", { sha256s: models.map(m => m.blob.sha256), baseUrl: this.baseUrl, userId: this.userId, userHash: this.userHash, asZip: asZip });
    }

    async getAppDataDir(): Promise<string> {
        return this.appDataDir
    }

    async openDataDirPicker(): Promise<string | null> {
        return await open({
            multiple: false,
            directory: true,
        })
    }

    async openCustomSlicerPicker(): Promise<string | null> {
        return await open({
            multiple: false,
            directory: false,
        });
    }

    async getMaxParallelism(): Promise<number> {
        return this.maxParallelism;
    }
}