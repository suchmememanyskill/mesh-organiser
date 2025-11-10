import { invoke } from "@tauri-apps/api/core";
import type { IThumbnailApi } from "../shared/thumbnail_api";

export class ThumbnailApi implements IThumbnailApi {
    async generateAllThumbnails(): Promise<void> {
        await invoke("update_images", { overwrite: true });
    }

    async generateMissingThumbnails(): Promise<void> {
        await invoke("update_images", { overwrite: false });
    }
}