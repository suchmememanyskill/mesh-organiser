import type { Model } from "./model_api";

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