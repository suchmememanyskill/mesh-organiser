import { invoke } from "@tauri-apps/api/core";
import type { Model } from "../shared/model_api";
import type { ISlicerApi, SlicerEntry } from "../shared/slicer_api";

export class SlicerApi implements ISlicerApi {
    async openInSlicer(models: Model[]): Promise<void> {
        await invoke("open_in_slicer", { modelIds: models.map(m => m.id) });
    }

    async availableSlicers(): Promise<SlicerEntry[]> {
        return await invoke("get_slicers", {});
    }
}