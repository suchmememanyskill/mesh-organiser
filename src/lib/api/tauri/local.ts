import { invoke } from "@tauri-apps/api/core";
import type { ILocalApi } from "../shared/services/local_api";
import type { Model } from "../shared/services/model_api";
import { open } from "@tauri-apps/plugin-dialog";
import { join } from "@tauri-apps/api/path";
import { openPath } from "@tauri-apps/plugin-opener";

export class LocalApi implements ILocalApi {
    appDataDir : string;
    maxParallelism : number;

    constructor(appDataDir: string, maxParallelism: number) {
        this.appDataDir = appDataDir;
        this.maxParallelism = maxParallelism;
    }

    async openInFolder(models: Model[]): Promise<void> {
        await invoke("open_in_folder", { modelIds: models.map(m => m.id) });
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

    async openCustomCss(): Promise<void> {
        const filePath = await join(
            this.appDataDir,
            "custom.css"
        );

        await openPath(filePath);
    }

    async getMaxParallelism(): Promise<number> {
        return this.maxParallelism;
    }
}