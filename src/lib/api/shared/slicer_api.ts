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
            return "prusaslicer://open/?file=";
        case "Cura":
            return "cura://open/?file=";
        case "Bambu Studio":
            return "bambustudio://open/?file=";
        case "OrcaSlicer":
            return "orcaslicer://open/?file=";
        case "Mesh Organiser":
            return "meshorganiser://open/?file=";
        default:
            return null;
    }
}

export class DefaultSlicerApi implements ISlicerApi {
    async openInSlicer(models: Model[]): Promise<void> {
        if (models.length === 0) {
            return;
        }

        if ( models.length >= 2) {
            toast.error("Opening multiple models in slicer is not supported in browser.");
            return;
        }

        let model = models[0];
        let modelUrl : string = (model.blob as any)._modelUrl;

        const link = document.createElement("a");
        let deepLink = slicerNameToDeepLink("PrusaSlicer");

        if (deepLink === null) {
            return;
        }

        deepLink += encodeURIComponent(modelUrl);

        link.href = deepLink;
        link.click();
        link.remove();
    }

    async availableSlicers(): Promise<SlicerEntry[]> {
        return [
            { slicer: "PrusaSlicer", installed: true },
            { slicer: "Cura", installed: true },
            { slicer: "Bambu Studio", installed: true },
            { slicer: "OrcaSlicer", installed: true },
            { slicer: "Mesh Organiser", installed: true },
        ];
    }
}
