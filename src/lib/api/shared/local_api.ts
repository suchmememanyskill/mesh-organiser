import type { Model } from "./model_api";

export const ILocalApi = Symbol('ILocalApi');

export interface ILocalApi {
    openInFolder(models : Model[], asZip: boolean) : Promise<void>;
    getAppDataDir() : Promise<string>;
    openDataDirPicker() : Promise<string|null>;
    openCustomSlicerPicker() : Promise<string|null>;
    getMaxParallelism() : Promise<number>;
}