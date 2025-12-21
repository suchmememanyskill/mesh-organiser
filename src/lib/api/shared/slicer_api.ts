import { configuration } from "$lib/configuration.svelte";
import type { IBlobApi } from "./blob_api";
import type { Model } from "./model_api";
import { toast } from "svelte-sonner";

export interface SlicerEntry
{
    slicer : string,
    installed : boolean,
}

export const ISlicerApi = Symbol('ISlicerApi');

export interface ISlicerApi {
    openInSlicer(models : Model[]) : Promise<void>;
    availableSlicers() : Promise<SlicerEntry[]>;
}

function slicerNameToDeepLink(slicerName: string): string | null {
    switch (slicerName) {
        case "PrusaSlicer":
            return "prusaslicer://open?file=";
        case "Cura":
            return "cura://open?file=";
        case "Bambu Studio":
            return "bambustudio://open?file=";
        case "OrcaSlicer":
            return "orcaslicer://open?file=";
        case "Mesh Organiser":
            return "meshorganiser://open?file=";
        default:
            return null;
    }
}

export class DefaultSlicerApi implements ISlicerApi {
    private blobApi : IBlobApi;

    constructor(blobApi : IBlobApi) {
        this.blobApi = blobApi;
    }

    async openInSlicer(models: Model[]): Promise<void> {
        let modelUrl;

        console.log(models);

        if (models.length === 0) {
            return;
        }
        else if (models.length === 1) {
            modelUrl = await this.blobApi.getBlobDownloadUrl(models[0].blob);
        }
        else if (models.length > 1) {
            modelUrl = await this.blobApi.getBlobsDownloadUrl(models.map(m => m.blob));
        }

        let deepLink = slicerNameToDeepLink(configuration.slicer ?? "OrcaSlicer");

        if (deepLink === null) {
            return;
        }

        deepLink += encodeURIComponent(modelUrl!);

        const link = document.createElement("a");
        link.href = deepLink;
        link.click();
        link.remove();
    }

    async availableSlicers(): Promise<SlicerEntry[]> {
        return [
            { slicer: "PrusaSlicer", installed: true },
            { slicer: "Cura", installed: true },
            { slicer: "OrcaSlicer", installed: true },
            { slicer: "Mesh Organiser", installed: true },
        ];
    }
}
